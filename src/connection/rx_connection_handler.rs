use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;

use log::{debug, error, info, trace, warn};
use metered::{*};
use nameof::{name_of};
use serde::Serializer;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::model::control_packet::ControlPacket;
use crate::serdes::deserializer::error::ReadError;
use crate::serdes::mqtt_decoder::MqttDecoder;

#[derive(Debug)]
pub struct RxConnectionHandler {
    pub(crate) metrics: RxConnectionHandlerMetrics,
    pub(crate) client_handler: Arc<RxClientHandler>,
}

#[metered(registry = RxConnectionHandlerMetrics)]
impl RxConnectionHandler {
    pub async fn handle_incoming_connections(&self, listener2broker: Sender<(SocketAddr, ControlPacket)>, stream_repository: Arc<DashMap<SocketAddr, OwnedWriteHalf>>) {
        trace!("MQTTListener::process");
        info!("Starting TCP Listener on port {}", 1883);
        let address = SocketAddr::from(([127, 0, 0, 1], 1883));
        let listener_instance = TcpListener::bind(address).await
            .unwrap_or_else(|error| {
                panic!("Cannot bind TCP Listener to {:?}. {:?}", address, error);
            });

        loop {
            match listener_instance.accept().await {
                Ok((stream, socket)) => {
                    info!("New connection request from {:?}", socket);

                    let handle = self.client_handler.clone();
                    let (in_stream, out_stream) = stream.into_split();
                    let stream_repository = stream_repository.clone();
                    let listener2broker = listener2broker.clone();
                    stream_repository.insert(socket, out_stream);
                    handle.handle_client(&socket, in_stream, listener2broker.clone()).await;
                }
                Err(error) => {
                    error!("Can't handle TCP Stream {:?}", error);
                }
            }
        }
    }

    pub fn new() -> Self {
        Self { metrics: RxConnectionHandlerMetrics::default(), client_handler: Arc::new(RxClientHandler::default()) }
    }
}

#[derive(Default, Debug)]
pub struct RxClientHandler {
    pub(crate) decoder: MqttDecoder,
    pub(crate) metrics: RxClientHandlerMetrics,

}

#[metered(registry = RxClientHandlerMetrics)]
impl RxClientHandler {
    #[measure([HitCount, InFlight, ResponseTime])]
    async fn handle_client(&self, socket: &SocketAddr, mut in_stream: OwnedReadHalf, listener2broker: Sender<(SocketAddr, ControlPacket)>) {
        info!("START - handle_client({})", socket);
        let socket = socket.clone();
        let decode = self.decoder.clone();
        let stream = Arc::new(Mutex::new(in_stream));
        tokio::spawn(async move {
            loop {
                match decode.decode_packet(stream.clone()).await {
                    Ok(control_packet) => {
                        debug!("Got new Control Packet from client: {:?}", socket);
                        match listener2broker.send((socket.clone(), control_packet)).await {
                            Ok(_) => {
                                debug!("Sent message to broker");
                                Ok(())
                            }
                            Err(err) => {
                                Err(format!("Can't send message to broker: {:?}", err))
                            }
                        }.expect("panic send_to_broker");
                    }
                    Err(err) => {
                        error!("Can't read any valid control packet from stream: {:?}", err);
                        match err.cause() {
                            ReadError::ConnectionError => {
                                warn!("Connection closed for client {:?}. Going to stop incoming messages handler.", socket);
                                break;
                            }
                            _ => {}
                        }
                    }
                };
            }
        });
        info!("END - handle_client({})", socket);
    }
}


