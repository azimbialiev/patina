use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use bitreader::BitReader;
use bytes::BytesMut;

use log::{debug, error, info, trace, warn};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use tokio::sync::mpsc::Sender;
use metered::{metered, Throughput, HitCount, InFlight, ResponseTime};
use nameof::{name_of, name_of_type};
use serde::Serializer;
use tokio::io::AsyncReadExt;

use crate::serdes::decoder::{DecodeError, Decoder, DecodeResult, FixedHeaderDecoder, PayloadDecoder, ReadError, ReadResult, VariableHeaderDecoder};
use crate::serdes::mqtt::{ControlPacket, ControlPacketType, FixedHeader};
use crate::serdes::mqtt_decoder::MqttDecoder;

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct RxConnectionHandler {
    pub(crate) metrics: RxConnectionHandlerMetrics,
    pub(crate) decoder: MqttDecoder,
}

#[metered(registry = RxConnectionHandlerMetrics)]
impl RxConnectionHandler {
    pub async fn handle_incoming_connections(&self, listener2broker: Sender<(SocketAddr, ControlPacket)>, stream_repository: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>) {
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
                    let listener2broker = listener2broker.clone();
                    let (in_stream, out_stream) = stream.into_split();
                    trace!("Acquiring {} lock", name_of!(stream_repository));
                    stream_repository.lock().await.insert(socket, out_stream);
                    self.handle_client(&socket, in_stream, listener2broker).await;
                }
                Err(error) => {
                    error!("Can't handle TCP Stream {:?}", error);
                }
            }
        }
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    async fn handle_client(&self, socket: &SocketAddr, in_stream: OwnedReadHalf, listener2broker: Sender<(SocketAddr, ControlPacket)>) {
        let socket = socket.clone();
        let decode = self.decoder.clone();
        tokio::spawn(async move {
            info!("{}::{}({})", name_of_type!(RxConnectionHandler), "handle_client", socket);
            let stream = Mutex::new(in_stream);
            loop {
                trace!("Acquiring {} lock", name_of!(stream));
                let stream = stream.lock().await;
                match decode.decode_packet(stream).await {
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
    }


}


