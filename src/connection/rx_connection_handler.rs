use std::net::SocketAddr;
use std::sync::Arc;

use dashmap::DashMap;
use log::{debug, error, info, trace, warn};
use metered::{*};
use tokio::io::BufReader;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::model::control_packet::ControlPacket;
use crate::serdes::deserializer::error::ReadError;
use crate::serdes::mqtt_decoder::MqttDecoder;

#[derive(Debug)]
pub struct RxConnectionHandler {
    pub(crate) metrics: RxConnectionHandlerMetrics,
    pub(crate) rx_client_handler: Arc<RxClientHandler>,
}

#[metered(registry = RxConnectionHandlerMetrics)]
impl RxConnectionHandler {
    //#[tokio::main(flavor = "multi_thread")]
    #[tokio::main(flavor = "multi_thread", worker_threads = 8)]
    //#[tokio::main(flavor = "current_thread")]
    pub async fn handle_incoming_connections(&self, listener2broker: Arc<Sender<(SocketAddr, ControlPacket)>>, stream_repository: Arc<DashMap<SocketAddr, OwnedWriteHalf>>) -> Result<(), Box<dyn std::error::Error>> {
        trace!("MQTTListener::process");
        info!("Starting TCP Listener on port {}", 1883);
        let address = SocketAddr::from(([0, 0, 0, 0], 1883));
        let listener_instance = TcpListener::bind(address).await
            .unwrap_or_else(|error| {
                panic!("Cannot bind TCP Listener to {:?}. {:?}", address, error);
            });
        let rx_client_handler = self.rx_client_handler.clone();
        listener_instance.set_ttl(240);
        info!("Spawned TcpListener listener poller");
        loop {
            match listener_instance
                .accept().await {
                Ok((stream, socket)) => {
                    info!("New connection request from {:?}", socket);

                    let rx_client_handler = rx_client_handler.clone();
                    let (in_stream, out_stream) = stream.into_split();
                    let stream_repository = stream_repository.clone();
                    let listener2broker = listener2broker.clone();
                    tokio::spawn(async move {
                        stream_repository.insert(socket, out_stream);
                        rx_client_handler.handle_client(&socket, in_stream, listener2broker.clone()).await;
                    });
                }
                Err(error) => {
                    error!("Can't handle TCP Stream {:?}", error);
                }
            }
        }

        Ok(())
    }

    pub fn new() -> Self {
        Self { metrics: RxConnectionHandlerMetrics::default(), rx_client_handler: Arc::new(RxClientHandler::default()) }
    }
}

#[derive(Default, Debug)]
pub struct RxClientHandler {
    pub(crate) decoder: Arc<MqttDecoder>,
    pub(crate) metrics: RxClientHandlerMetrics,

}

#[metered(registry = RxClientHandlerMetrics)]
impl RxClientHandler {

    #[measure([HitCount, InFlight, ResponseTime])]
    async fn handle_client(&self, socket: &SocketAddr,mut in_stream: OwnedReadHalf, listener2broker: Arc<Sender<(SocketAddr, ControlPacket)>>) {
        debug!("START - handle_client({})", socket);
        let socket = socket.clone();
        let decode = self.decoder.clone();
        loop {
            match decode.decode_packet(in_stream).await {
                Ok((ret_stream, control_packet)) => {
                    in_stream = ret_stream;
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
                    break;
                }
            };
        }

        debug!("END - handle_client({})", socket);
    }
}


