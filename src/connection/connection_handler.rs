use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use crate::connection::connection::{encode_packet, read_packet, write_buffer};

use crate::serdes::decoder::ReadError;
use crate::serdes::mqtt::{ControlPacket, ControlPacketType};

#[derive(Debug)]
pub struct ConnectionHandler {}


impl ConnectionHandler {
    pub async fn handle_outgoing_connections(mut broker2listener: mpsc::Receiver<(SocketAddr, ControlPacket)>, client2write_half: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>) {
        loop {
            match broker2listener.recv().await {
                None => {}
                Some((socket, packet)) => {
                    if packet.fixed_header().packet_type() == ControlPacketType::DISCONNECT {
                        debug!("Handling disconnection for client {:?}", socket);
                        let mut lock = client2write_half.lock().await;
                        let client_tx = lock.get_mut(&socket).expect("panic client2write_half");
                        client_tx.shutdown().await.expect("panic shutdown write half");
                        lock.remove(&socket);
                        continue;
                    }

                    debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(),socket);
                    match encode_packet(&packet) {
                        Ok(buffer) => {
                            trace!("Successfully encoded packet");
                            let mut lock = client2write_half.lock().await;
                            let client_tx = lock.get_mut(&socket).expect("panic client2write_half");
                            match write_buffer(&buffer, client_tx).await {
                                Ok(_) => {
                                    trace!("Successfully sent packet");
                                }
                                Err(err) => {
                                    error!("Can't send data to client {:?}: {:?}", socket, err);
                                    continue;
                                }
                            }
                        }
                        Err(err) => {
                            error!("Can't encode Control Packet: {:?}", err);
                            //return Err(format!("Can't encode Control Packet: {:?}", err));
                        }
                    };
                }
            }
        }
    }

    pub async fn handle_incoming_connections(port: u16, listener2broker: Sender<(SocketAddr, ControlPacket)>,
                                             rx_clone: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>,
    ) {
        trace!("MQTTListener::process");
        info!("Starting TCP Listener on port {}", port);
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener_instance = TcpListener::bind(address).await
            .unwrap_or_else(|error| {
                panic!("Cannot bind TCP Listener to {:?}. {:?}", address, error);
            });

        loop {
            match listener_instance.accept().await {
                Ok((stream, client)) => {
                    debug!("New connection request from {:?}", client);
                    debug!("Spin up incoming packet handler");

                    let listener2broker_ = listener2broker.clone();
                    let (client_rx, client_tx) = stream.into_split();
                    rx_clone.lock().await.insert(client, client_tx);
                    let client_rx_ = Mutex::new(client_rx);
                    tokio::spawn(async move {
                        //let stream_ = stream.try_clone();
                        loop {
                            match read_packet(client_rx_.lock().await).await {
                                Ok(control_packet) => {
                                    debug!("Got new Control Packet from client: {:?}", client);
                                    match listener2broker_.send((client.clone(), control_packet)).await {
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
                                            warn!("Connection closed for client {:?}. Going to stop incoming messages handler.", client);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            };
                        }
                    });
                }
                Err(error) => {
                    error!("Can't handle TCP Stream {:?}", error);
                }
            }
        }
    }
}
