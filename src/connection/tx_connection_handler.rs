use core::fmt;
use std::borrow::BorrowMut;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use bytes::BytesMut;
use dashmap::DashMap;
use log::{debug, error, trace};
use metered::{*};
use nameof::name_of;
use serde::Serializer;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::Sender;

use crate::model::control_packet::ControlPacket;
use crate::model::fixed_header::ControlPacketType;
use crate::model::reason_code::ReasonCode;
use crate::serdes::mqtt_encoder::MqttEncoder;

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct TxConnectionHandler {
    pub(crate) metrics: TxConnectionHandlerMetrics,
    pub(crate) client_handler: TxClientHandler,
}

#[metered(registry = TxConnectionHandlerMetrics)]
impl TxConnectionHandler {
    pub async fn handle_outgoing_connections(&self, mut broker2listener: mpsc::Receiver<(SocketAddr, ControlPacket)>, listener2broker: Sender<(SocketAddr, ControlPacket)>, stream_repository: Arc<DashMap<SocketAddr, OwnedWriteHalf>>) {
        loop {
            if let Some((socket, packet)) = broker2listener.recv().await {
                let handler = self.client_handler.clone();
                let mut stream_repository = stream_repository.clone();
                let listener2broker = listener2broker.clone();
                tokio::spawn(async move {
                    trace!("Acquiring {} lock", name_of!(stream_repository));
                    if Self::is_disconnection(&packet).await {
                        debug!("Handling disconnection for socket {:?}", socket);
                        if let Some(mut out_stream) = stream_repository.get_mut(&socket){
                            match out_stream.borrow_mut().shutdown().await {
                                Ok(_) => {
                                    debug!("Socket {:?} shutdown", socket);
                                }
                                Err(err) => {
                                    error!("Can't shutdown socket {}. {}", socket, err);
                                }
                            }
                        }
                        stream_repository.remove(&socket);
                    } else {
                        debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(), socket);

                        if let Some(mut out_stream) = stream_repository.get_mut(&socket){
                            let mut out_stream = out_stream.borrow_mut();
                            match handler.send_packet(&socket, &packet, out_stream).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Can't send packet {:?} to socket {}. Going to propagate disconnection.", packet.fixed_header().packet_type(), socket);
                                    match listener2broker.send((socket, ControlPacket::disconnect(ReasonCode::UnspecifiedError))).await {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Can't send packet to broker. {}", err);
                                        }
                                    };
                                }
                            }
                        }
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
    async fn send_packet(&self, socket: &SocketAddr, packet: &ControlPacket, stream: &mut OwnedWriteHalf) -> Result<(), WriteError> {
        match self.encoder.encode_packet(packet) {
            Ok(buffer) => {
                trace!("Successfully encoded packet");
                match self.write_buffer(&buffer, stream).await {
                    Ok(_) => {
                        trace!("Successfully sent packet");
                        Ok(())
                    }
                    Err(err) => {
                        error!("Can't send data to client {:?}: {:?}", socket, err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                error!("Can't encode Control Packet: {:?}", err);
                Err(WriteError::EncodeError)
            }
        }
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
                trace!("Can't write packets to stream: {:?}", e);
                return Err(WriteError::SendError);
            }
        };
        //Ok(())
        match stream.flush().await {
            Ok(_) => { Ok(()) }
            Err(e) => {
                trace!("Can't flush buffered writer: {:?}", e);
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
