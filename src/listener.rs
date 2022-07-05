use std::borrow::{BorrowMut};
use std::net::SocketAddr;
use tokio::net::{TcpListener};
use log::{trace, info, debug, error, warn};
use tokio::sync::{broadcast, RwLock};
use tokio::sync::mpsc;
use crate::mqtt::{ControlPacket, ControlPacketType, FixedHeader};
use crate::connection::{encode_packet, read_packet, write_buffer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::decoder::{ReadError};
use crate::encoder::EncodeResult;

use metrics::{gauge, register_gauge, register_counter, increment_counter, increment_gauge, decrement_gauge};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::SendError;

pub static LISTENER_RECEIVED_PACKETS_COUNT: &str = "listener.received_packets.count";
pub static LISTENER_INCOMING_HANDLER_THREADS_COUNT: &str = "listener.incoming_handler_threads.count";


#[derive(Debug)]
pub struct Listener {
    port: u16,
    listener_instance: TcpListener,
    listener2broker_packet_tx: mpsc::Sender<(SocketAddr, ControlPacket)>,
    listener2broker_client_tx_tx: mpsc::Sender<(SocketAddr, OwnedWriteHalf)>,

}

impl Listener {
    pub async fn new(port: u16, listener2broker_packet_tx: mpsc::Sender<(SocketAddr, ControlPacket)>, listener2broker_client_tx_tx: mpsc::Sender<(SocketAddr, OwnedWriteHalf)>) -> Listener {
        trace!("MQTTListener::new()");

        register_counter!(LISTENER_RECEIVED_PACKETS_COUNT);
        register_gauge!(LISTENER_INCOMING_HANDLER_THREADS_COUNT);

        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener_instance = TcpListener::bind(address).await
            .unwrap_or_else(|error| {
                panic!("Cannot bind TCP Listener to {:?}. {:?}", address, error);
            });
        debug!("Spin up outgoing packet handle");
        Listener {
            port,
            listener_instance,
            listener2broker_packet_tx,
            listener2broker_client_tx_tx,
        }
    }

    pub async fn process(&mut self) {
        trace!("MQTTListener::process");
        info!("Starting TCP Listener on port {}", self.port);
        loop {
            match self.listener_instance.accept().await {
                Ok((stream, client)) => {
                    debug!("New connection request from {:?}", client);

                    let (client_rx, client_tx) = stream.into_split();

                    match self.listener2broker_client_tx_tx.send((client, client_tx)).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Can't send clients's {:?} outgoing connection to broker: {:?}", client, err);
                        }
                    };
                    let listener2broker_packet_tx = self.listener2broker_packet_tx.clone();
                    debug!("Spin up incoming packet handler");
                    tokio::spawn(async move {
                        increment_gauge!(LISTENER_INCOMING_HANDLER_THREADS_COUNT, 1.0);
                        match handle_incoming_packets(listener2broker_packet_tx, client_rx, client).await {
                            _ => {
                                decrement_gauge!(LISTENER_INCOMING_HANDLER_THREADS_COUNT, 1.0);
                            }
                        };
                    });
                }
                Err(error) => {
                    error!("Can't handle TCP Stream {:?}", error);
                }
            }
        }
    }
}

async fn handle_incoming_packets<'a>(listener2broker_tx: mpsc::Sender<(SocketAddr, ControlPacket)>, mut stream: OwnedReadHalf, client: SocketAddr) -> Result<(), ()> {
    trace!("MQTTListener::handle_incoming_packet");
    loop {
        debug!("Reading incoming packets for client {:?}", client);
        let result = match read_packet(stream.borrow_mut()).await {
            Ok(control_packet) => {
                debug!("Got new Control Packet from client: {:?}", client);
                increment_counter!(LISTENER_RECEIVED_PACKETS_COUNT);
                send_to_broker(listener2broker_tx.clone(), client, control_packet).await
            }
            Err(err) => {
                error!("Can't read any valid control packet from stream: {:?}", err);
                match err.cause() {
                    ReadError::ConnectionError => {
                        warn!("Connection closed for client {:?}. Going to stop incoming messages handler.", client);
                    }
                    _ => {}
                }
                // let disconnect_packet = ControlPacket::new(FixedHeader::new(ControlPacketType::DISCONNECT, vec![false, false, false, false], 0), None, None);
                // send_to_broker(listener2broker_tx.clone(), client, disconnect_packet).await;
                return Err(());
            }
        };

        match result {
            Err(err) => {
                error!("Can't handle incoming packet from {:?}", client);
                return Err(());
            }
            _ => {}
        }
    }
}

async fn send_to_broker(listener2broker_tx: mpsc::Sender<(SocketAddr, ControlPacket)>, client: SocketAddr, control_packet: ControlPacket) -> Result<(), String> {
    return match listener2broker_tx.send((client.clone(), control_packet)).await {
        Ok(_) => {
            debug!("Sent message to broker");
            Ok(())
        }
        Err(err) => {
            Err(format!("Can't send message to broker: {:?}", err))
        }
    };
}
