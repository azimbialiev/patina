use core::fmt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use bytes::BytesMut;
use log::{debug, error, trace};
use metered::{*};
use nameof::name_of;
use serde::Serializer;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{mpsc, Mutex};

use crate::model::control_packet::ControlPacket;
use crate::model::fixed_header::ControlPacketType;
use crate::serdes::mqtt_encoder::MqttEncoder;

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct TxConnectionHandler {
    pub(crate) metrics: TxConnectionHandlerMetrics,
    pub(crate) client_handler: TxClientHandler,
}

#[metered(registry = TxConnectionHandlerMetrics)]
impl TxConnectionHandler {
    pub async fn handle_outgoing_connections(&self, mut broker2listener: mpsc::Receiver<(SocketAddr, ControlPacket)>, stream_repository: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>) {
        loop {
            if let Some((socket, packet)) = broker2listener.recv().await {
                let handler = self.client_handler.clone();
                let mut stream_repository = stream_repository.clone();
                tokio::spawn(async move {
                    trace!("Acquiring {} lock", name_of!(stream_repository));
                    let mut stream_repository = stream_repository.lock().await;
                    let client_tx = stream_repository.get_mut(&socket).expect("panic client2write_half");
                    if Self::is_disconnection(&packet).await {
                        debug!("Handling disconnection for client {:?}", socket);
                        client_tx.shutdown().await.expect("panic shutdown write half");
                        stream_repository.remove(&socket);
                    } else {
                        debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(), socket);
                        handler.send_packet(&socket, &packet, client_tx).await
                    }
                });
            }
        }
    }

    async fn is_disconnection(packet: &ControlPacket) -> bool {
        if packet.fixed_header().packet_type() == ControlPacketType::DISCONNECT {
            return true;
        }
        return false;
    }
}


#[derive(Default, Clone, Debug)]
pub struct TxClientHandler(Arc<TxClientHandlerImpl>);

impl Deref for TxClientHandler {
    type Target = TxClientHandlerImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl serde::Serialize for TxClientHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        return self.deref().serialize(serializer);
    }
}

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct TxClientHandlerImpl {
    pub(crate) encoder: MqttEncoder,
    pub(crate) metrics: TxClientHandlerMetrics,

}

#[metered(registry = TxClientHandlerMetrics)]
impl TxClientHandlerImpl {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    async fn send_packet(&self, socket: &SocketAddr, packet: &ControlPacket, stream: &mut OwnedWriteHalf) {
        match self.encoder.encode_packet(packet) {
            Ok(buffer) => {
                trace!("Successfully encoded packet");
                match self.write_buffer(&buffer, stream).await {
                    Ok(_) => {
                        trace!("Successfully sent packet");
                    }
                    Err(err) => {
                        error!("Can't send data to client {:?}: {:?}", socket, err);
                    }
                }
            }
            Err(err) => {
                error!("Can't encode Control Packet: {:?}", err);
                //return Err(format!("Can't encode Control Packet: {:?}", err));
            }
        };
    }


    #[measure([Throughput, ResponseTime])]
    pub async fn write_buffer(&self, buffer: &BytesMut, stream: &mut OwnedWriteHalf) -> WriteResult {
        debug!("MQTTConnection::write");
        trace!("Buffer Length: {:?}", buffer.len());
        for i in buffer.clone() {
            trace!("Going to write: {:#04X?}", i );
        }

        match stream.write(buffer).await {
            Ok(result) => {
                trace!("{:?} bytes written to stream", result);
            }
            Err(e) => {
                error!("Can't send packet: {:?}", e);
                return Err(WriteError::SendError);
            }
        };
        //Ok(())
        match stream.flush().await {
            Ok(_) => { Ok(()) }
            Err(e) => {
                error!("Can't flush buffered writer: {:?}", e);
                return Err(WriteError::FlushError);
            }
        }
    }
}

pub type WriteResult = Result<(), WriteError>; //TODO Needs better errors

#[derive(Debug, PartialEq, Clone)]
pub enum WriteError {
    ConnectionTimedOut,
    EncodeError,
    SendError,
    FlushError,
}

impl fmt::Display for WriteError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //self.description().fmt(fmt)
        match *self {
            WriteError::ConnectionTimedOut => write!(fmt, "WriteError::ConnectionTimedOut"),
            WriteError::EncodeError => write!(fmt, "WriteError::EncodeError"),
            WriteError::SendError => write!(fmt, "WriteError::SendError"),
            WriteError::FlushError => write!(fmt, "WriteError::FlushError"),
        }
    }
}
