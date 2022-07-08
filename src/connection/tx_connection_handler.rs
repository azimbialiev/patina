use core::fmt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use bytes::BytesMut;

use log::{debug, error, trace};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{mpsc, Mutex};
use metered::{metered, Throughput, HitCount, InFlight, ResponseTime};
use nameof::name_of;
use crate::serdes::encoder::{Encoder, EncodeResult, FixedHeaderEncoder, LengthCalculator, OptEncoder, PayloadEncoder, VariableHeaderEncoder};

use crate::serdes::mqtt::{ControlPacket, ControlPacketType};

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct TxConnectionHandler {
    pub(crate) metrics: TxConnectionHandlerMetrics,
}

#[metered(registry = TxConnectionHandlerMetrics)]
impl TxConnectionHandler {
    pub async fn handle_outgoing_connections(&self, mut broker2listener: mpsc::Receiver<(SocketAddr, ControlPacket)>, stream_repository: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>) {
        loop {
            if let Some((socket, packet)) = broker2listener.recv().await {
                trace!("Acquiring {} lock", name_of!(stream_repository));
                let mut stream_repository = stream_repository.lock().await;
                let client_tx = stream_repository.get_mut(&socket).expect("panic client2write_half");
                if Self::is_disconnection(&packet).await {
                    debug!("Handling disconnection for client {:?}", socket);
                    client_tx.shutdown().await.expect("panic shutdown write half");
                    stream_repository.remove(&socket);
                } else {
                    debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(), socket);
                    self.send_packet(&socket, &packet, client_tx).await
                }
            }
        }
    }

    async fn is_disconnection(packet: &ControlPacket) -> bool {
        if packet.fixed_header().packet_type() == ControlPacketType::DISCONNECT {
            return true;
        }
        return false;
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    async fn send_packet(&self, socket: &SocketAddr, packet: &ControlPacket, stream: &mut OwnedWriteHalf) {
        match self.encode_packet(packet) {
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

    pub fn encode_packet(&self, packet: &ControlPacket) -> EncodeResult<BytesMut> {
        debug!("Connection::encode_packet");
        trace!("Encoding packet: {:?} - {:?}", packet.fixed_header().packet_type(), packet);
        let mut calculated_remaining_length = 0;

        let mut payload_encoder = PayloadEncoder::new(packet.fixed_header().packet_type());
        match packet.payload_opt() {
            None => {}
            Some(payload) => {
                let remaining_length = payload_encoder.calculate_length(payload) as u64;
                trace!("Payload Length: {:?}", remaining_length);
                calculated_remaining_length = calculated_remaining_length + remaining_length;
            }
        }

        let mut variable_header_encoder = VariableHeaderEncoder::new(packet.fixed_header().packet_type());
        match packet.variable_header_opt() {
            None => {}
            Some(variable_header) => {
                let remaining_length = variable_header_encoder.calculate_length(variable_header) as u64;
                trace!("Variable Header Length: {:?}", remaining_length);
                calculated_remaining_length = calculated_remaining_length + remaining_length;
            }
        }

        let mut fixed_header_encoder = FixedHeaderEncoder::new();
        let fixed_header_length = fixed_header_encoder.calculate_length(&(packet.fixed_header(), calculated_remaining_length));
        trace!("Fixed Header Length: {:?}", fixed_header_length);
        debug!("Control Packet Remaining Length: {:?}", calculated_remaining_length);
        let mut buffer = BytesMut::with_capacity(fixed_header_length + calculated_remaining_length as usize);
        fixed_header_encoder.encode(&(packet.fixed_header(), calculated_remaining_length), &mut buffer)?;
        variable_header_encoder.encode_opt(packet.variable_header_opt(), &mut buffer)?;
        payload_encoder.encode_opt(packet.payload_opt(), &mut buffer).expect("panic encode_opt");
        Ok(buffer)
    }

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
