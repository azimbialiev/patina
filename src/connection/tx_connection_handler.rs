use core::fmt;
use std::borrow::BorrowMut;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::BytesMut;
use dashmap::DashMap;
use log::{debug, error, trace};
use metered::{*};
use nameof::name_of;
use serde::Serializer;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::{ClientHandler, TopicHandler};

use crate::model::control_packet::ControlPacket;
use crate::model::fixed_header::ControlPacketType;
use crate::serdes::mqtt_encoder::MqttEncoder;

#[derive(Debug)]
pub struct TxConnectionHandler {
    pub(crate) metrics: TxConnectionHandlerMetrics,
    pub(crate) tx_client_handler: Arc<TxClientHandler>,
    client_handler: Arc<ClientHandler>,
    topic_handler: Arc<TopicHandler>,
    pub(crate) encoder: MqttEncoder,

}

#[metered(registry = TxConnectionHandlerMetrics)]
impl TxConnectionHandler {
    pub async fn handle_outgoing_connections(&self, mut broker2listener: Receiver<(Vec<SocketAddr>, ControlPacket)>, listener2broker: Sender<(SocketAddr, ControlPacket)>, stream_repository: Arc<DashMap<SocketAddr, OwnedWriteHalf>>) {
        loop {
            if let Some((sockets, packet)) = broker2listener.recv().await {
                let encoder = self.encoder.clone();
                let tx_client_handler = self.tx_client_handler.clone();
                let client_handler = self.client_handler.clone();
                let topic_handler = self.topic_handler.clone();
                let mut stream_repository = stream_repository.clone();
                let listener2broker = listener2broker.clone();
                tokio::spawn(async move {
                    let packet = Arc::new(packet);
                    match encoder.encode_packet(&packet) {
                        Ok(encoded_packet) => {
                            let encoded_packet = Arc::new(encoded_packet);
                            for socket in sockets {
                                let encoded_packet = encoded_packet.clone();
                                let packet = packet.clone();
                                let tx_client_handler = tx_client_handler.clone();
                                let client_handler = client_handler.clone();
                                let topic_handler = topic_handler.clone();
                                let mut stream_repository = stream_repository.clone();
                                let listener2broker = listener2broker.clone();

                                tokio::spawn(async move {
                                    trace!("Acquiring {} lock", name_of!(stream_repository));
                                    if Self::is_disconnection(&packet).await {
                                        debug!("Handling disconnection for socket {:?}", socket);
                                        Self::clean_after_disconnection(&socket, &stream_repository, &client_handler, &topic_handler).await;
                                    } else {
                                        debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(), socket);

                                        if let Some(mut out_stream) = stream_repository.get_mut(&socket) {
                                            let mut out_stream = out_stream.borrow_mut();
                                            match tx_client_handler.send_packet(&socket, &encoded_packet.clone(), out_stream).await {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    error!("Can't send packet {:?} to socket {}. {}", packet.fixed_header().packet_type(), socket, err);
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        Err(err) => {
                            panic!("Can't encode Control Packet: {:?}", err);
                        }
                    }
                });
            }
        }
    }

    async fn clean_after_disconnection(socket: &SocketAddr, stream_repository: &Arc<DashMap<SocketAddr, OwnedWriteHalf>>, client_handler: &Arc<ClientHandler>, topic_handler: &Arc<TopicHandler>) {
        debug!("clean_after_disconnection");
        if let Some(client_id) = client_handler.unregister_by_socket(socket) {
            topic_handler.unsubscribe_all(&client_id);
        }
        if let Some(mut out_stream) = stream_repository.get_mut(&socket) {
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
    }

    async fn is_disconnection(packet: &ControlPacket) -> bool {
        if packet.fixed_header().packet_type() == ControlPacketType::DISCONNECT {
            return true;
        }
        return false;
    }

    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>) -> Self {
        Self { metrics: TxConnectionHandlerMetrics::default(), tx_client_handler: Arc::new(TxClientHandler::default()), client_handler, topic_handler, encoder: MqttEncoder::default() }
    }
}


#[derive(Default, Debug)]
pub struct TxClientHandler {
    pub(crate) metrics: TxClientHandlerMetrics,

}

#[metered(registry = TxClientHandlerMetrics)]
impl TxClientHandler {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    async fn send_packet(&self, socket: &SocketAddr, encoded_packet: &BytesMut, stream: &mut OwnedWriteHalf) -> Result<(), WriteError> {
        trace!("Successfully encoded packet");
        match self.write_buffer(encoded_packet, stream).await {
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


    #[measure([Throughput, ResponseTime])]
    pub async fn write_buffer(&self, buffer: &BytesMut, stream: &mut OwnedWriteHalf) -> WriteResult {
        debug!("MQTTConnection::write");
        trace!("Buffer Length: {:?}", buffer.len());
        match stream.try_write(buffer) {
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
