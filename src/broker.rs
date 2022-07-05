use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::net::SocketAddr;

use log::{trace, info, debug, warn, error};
use tokio::sync::{broadcast, RwLock, RwLockWriteGuard, Mutex};
use crate::mqtt::{ConnectAcknowledgeFlags, ControlPacket, ControlPacketType, FixedHeader, Payload, QoSLevel, ReasonCode, VariableHeader};
use tokio::sync::mpsc;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use chrono::{DateTime, Local};
use rand;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::topic_handler::TopicHandler;
use crate::session::{Session, SessionState};
use metrics::{gauge, register_gauge, register_counter, increment_gauge, decrement_gauge, increment_counter};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::time::Instant;
use crate::connection::{encode_packet, write_buffer, WriteResult};

use dashmap::{DashMap};

pub static BROKER_HANDLED_PACKETS_COUNT: &str = "broker.handled_packets.count";
pub static BROKER_CURRENT_HANDLER_THREADS_COUNT: &str = "broker.current_handler_threads.count";
pub static BROKER_PROCESS_MESSAGE_MICROS: &str = "broker.process_message.micros";

pub static BROKER_CONNECTED_CLIENTS_COUNT: &str = "broker.connected_clients.count";
pub static BROKER_SENT_PACKETS_COUNT: &str = "broker.sent_packets.count";

pub static BROKER_REGISTER_SOCKET_WAIT_MICROS: &str = "broker.register_socket.wait.micros";
pub static BROKER_ID2SOCKET_WAIT_COUNT: &str = "broker.id2socket.wait.count";
pub static BROKER_SOCKET2ID_WAIT_COUNT: &str = "broker.socket2id.wait.count";






lazy_static! {

    static  ref id2session: DashMap<String, Session> = {
        let map = DashMap::new();
        map
    };

    static ref socket2id: DashMap<SocketAddr, String> = {
        let map = DashMap::new();
        map
    };

    static ref id2socket: DashMap<String, SocketAddr> = {
        let map = DashMap::new();
        map
    };

    static ref socket2stream: DashMap<SocketAddr, OwnedWriteHalf> = {
        let map = DashMap::new();
        map
    };

    static ref topic_handler: TopicHandler = {
        TopicHandler::new()
    };
}


#[derive(Debug)]
pub struct Broker {}

impl Broker {
    pub fn new() -> Self {
        register_counter!(BROKER_HANDLED_PACKETS_COUNT);
        register_gauge!(BROKER_CURRENT_HANDLER_THREADS_COUNT);
        register_gauge!(BROKER_PROCESS_MESSAGE_MICROS);
        register_gauge!(BROKER_CONNECTED_CLIENTS_COUNT);
        register_counter!(BROKER_SENT_PACKETS_COUNT);

        register_gauge!(BROKER_REGISTER_SOCKET_WAIT_MICROS);
        register_gauge!(BROKER_ID2SOCKET_WAIT_COUNT);
        register_gauge!(BROKER_SOCKET2ID_WAIT_COUNT);

        Broker {}
    }


    pub async fn start(&'static self, mut listener2broker_packet_rx: mpsc::Receiver<(SocketAddr, ControlPacket)>, mut listener2broker_client_tx_rx: mpsc::Receiver<(SocketAddr, OwnedWriteHalf)>) {
        info!("Broker::start");
        tokio::spawn(async move {
            loop {
                match listener2broker_client_tx_rx.recv().await {
                    None => { warn!("Channel has been closed!"); }
                    Some((socket, stream)) => {
                        register_outgoing_stream(socket, stream).await;
                    }
                }
            }
        });

        loop {
            trace!("Next iteration");
            match listener2broker_packet_rx.recv().await {
                None => {
                    warn!("Channel has been closed!");
                }
                Some((client, control_packet)) => {
                    tokio::spawn(async move {
                        let now = Instant::now();

                        trace!("Spawned new thread");
                        increment_gauge!(BROKER_CURRENT_HANDLER_THREADS_COUNT, 1.0);
                        match process_message(client, control_packet).await {
                            _ => {
                                gauge!(BROKER_PROCESS_MESSAGE_MICROS, now.elapsed().as_micros() as f64);
                                increment_counter!(BROKER_HANDLED_PACKETS_COUNT);
                                decrement_gauge!(BROKER_CURRENT_HANDLER_THREADS_COUNT, 1.0);
                            }
                        };
                    });
                }
            }
        }
    }
}

async fn process_message(socket: SocketAddr, control_packet: ControlPacket) -> Result<(), ()> {
    debug!("Going to handle control packet: {:?} from client {:?}", control_packet.fixed_header().packet_type(), socket);

    match control_packet.fixed_header().packet_type() {
        ControlPacketType::RESERVED => {}
        ControlPacketType::CONNECT => {
            let mut client_id = generate_client_id();
            if control_packet.has_client_id() {
                debug!("Using client's client_id");
                client_id = control_packet.payload().unwrap().client_id().unwrap().to_string();
            }
            info!("CONNECT client: {:?}", client_id);
            register_socket(&client_id, socket);
            let mut session_present = false;
            if control_packet.variable_header().unwrap().connect_flags().clean_start_flag() {
                debug!("Creating clean session for client: {:?}", client_id);
                register_clean_session(&client_id);
                topic_handler.remove_all_subscriptions(&client_id);
            } else {
                session_present = match register_session(&client_id) {
                    SessionState::SessionPresent => true,
                    SessionState::CleanSession => false
                };
            }
            let connack_packet = ControlPacket::connack(session_present);
            send_packet(socket, connack_packet).await;
            //TODO Check Auth
            //TODO Check previous session using client_id
            //TODO Check clean_start
        }
        ControlPacketType::CONNACK => {}
        ControlPacketType::PUBLISH => {
            let client_id = get_client_id(&socket).await?;
            if control_packet.fixed_header().qos_level() == &QoSLevel::AtLeastOnce {
                trace!("Sending PUBACK for {:?} Packet Identifier to client {:?}", control_packet.variable_header().unwrap().packet_identifier(), client_id);
                let puback_packet = ControlPacket::puback(control_packet.variable_header().unwrap().packet_identifier());
                send_packet(socket, puback_packet).await;
            }
            if control_packet.fixed_header().qos_level() == &QoSLevel::ExactlyOnce {
                trace!("Sending PUBREC for {:?} Packet Identifier to client {:?}", control_packet.variable_header().unwrap().packet_identifier(), client_id);
                let pubrec_packet = ControlPacket::pubrec(control_packet.variable_header().unwrap().packet_identifier());
                send_packet(socket, pubrec_packet).await;
            }
            let topic_path = control_packet.variable_header().unwrap().topic_name();
            let subscribers = topic_handler.find_subscribers(topic_path);
            info!("PUBLISH client: {:?} to topic:{:?}. Subscribers count: {:?}", client_id, topic_path, subscribers.len());
            trace!("Found subscribers {:?} for topic {:?}", subscribers, topic_path);

            persist_packets(&subscribers, &control_packet);
            let clients = subscribers
                .iter()
                .map(|receiver| {
                    id2socket.get(receiver)
                })
                .filter(Option::is_some)
                .map(|c| c.unwrap().clone())
                .collect();
            send_packets(clients, control_packet).await;
        }
        ControlPacketType::PUBACK => {
            let client_id = get_client_id(&socket).await?;
            id2session.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
            // id2session.lock().await.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
        }
        ControlPacketType::PUBREC => {}
        ControlPacketType::PUBREL => {
            let client_id = get_client_id(&socket).await?;
            id2session.get_mut(&client_id).unwrap().register_pubrel(client_id.clone(), &control_packet);
            trace!("Sending PUBCOMP for {:?} Packet Identifier to client {:?}", control_packet.variable_header().unwrap().packet_identifier(), client_id);
            let pubcomp_packet = ControlPacket::pubcomp(control_packet.variable_header().unwrap().packet_identifier());
            send_packet(socket, pubcomp_packet).await;
        }
        ControlPacketType::PUBCOMP => {}
        ControlPacketType::SUBSCRIBE => {
            let client_id = get_client_id(&socket).await?;
            let topic_filters = control_packet.payload().unwrap().topic_filters();
            info!("SUBSCRIBE client: {:?} to topics: {:?}", client_id, topic_filters);

            let mut reason_codes = Vec::with_capacity(topic_filters.len());
            for topic_filter in topic_filters {
                topic_handler.add_subscriber(&client_id, topic_filter.topic_filter());
                reason_codes.push(ReasonCode::GrantedQoS0);
                debug!("Subscribed client {:?} to topic {:?}", client_id, topic_filter.topic_filter());
            }
            let suback_packet = ControlPacket::suback(control_packet.variable_header().unwrap().packet_identifier(), reason_codes);

            send_packet(socket, suback_packet).await;
        }
        ControlPacketType::SUBACK => {}
        ControlPacketType::UNSUBSCRIBE => {}
        ControlPacketType::UNSUBACK => {}
        ControlPacketType::PINGREQ => {
            let client_id = get_client_id(&socket).await?;
            debug!("PINGREQ from client {:?}", client_id);
            let pingresp_packet = ControlPacket::pingresp();
            send_packet(socket, pingresp_packet).await;
        }
        ControlPacketType::PINGRESP => {}
        ControlPacketType::DISCONNECT => {
            let client_id = get_client_id(&socket).await?;
            info!("Got a DISCONNECT packet for client {:?}. Going to clean outgoing connections", client_id);
            unregister_socket(&client_id, &socket);
            unregister_outgoing_stream(&socket).await;
        }
        ControlPacketType::AUTH => {}
    };
    Ok(())
}

async fn register_outgoing_stream(socket: SocketAddr, stream: OwnedWriteHalf) -> Option<OwnedWriteHalf> {
    trace!("Broker::register_outgoing_stream");
    info!("Registering outgoing connection for client: {:?}", socket);
    return socket2stream.insert(socket, stream);
}

async fn unregister_outgoing_stream(socket: &SocketAddr) -> Option<(SocketAddr, OwnedWriteHalf)> {
    trace!("Broker::unregister_outgoing_stream");
    info!("Unregistering outgoing connection for client: {:?}", socket);
    return socket2stream.remove(socket);
}

async fn get_client_id(socket: &SocketAddr) -> Result<String, ()> {
    trace!("Broker::get_client_id");
    return match socket2id.get(socket) {
        None => {
            Err(())
        }
        Some(client_id) => {
            Ok(client_id.clone())
        }
    };
}

fn unregister_socket(client_id: &String, socket: &SocketAddr) {
    let now = Instant::now();
    trace!("Broker::unregister_socket");
    increment_gauge!(BROKER_SOCKET2ID_WAIT_COUNT, 1.0);
    match socket2id.remove(socket) {
        None => {
            trace!("Unregister socket2id: {:?} -> {:?}", socket, client_id);
            decrement_gauge!(BROKER_CONNECTED_CLIENTS_COUNT, 1.0);
        }
        Some(_) => {
            error!("Need to handle 'session taken over' case");
        }
    };
    decrement_gauge!(BROKER_SOCKET2ID_WAIT_COUNT, 1.0);

    increment_gauge!(BROKER_ID2SOCKET_WAIT_COUNT, 1.0);
    match id2socket.remove(client_id) {
        None => {
            trace!("Unregister id2socket: {:?} -> {:?}", client_id, socket);
        }
        Some(previous_socket) => {
            error!("Need to handle 'session taken over' case");
        }
    };
    decrement_gauge!(BROKER_ID2SOCKET_WAIT_COUNT, 1.0);
    gauge!(BROKER_REGISTER_SOCKET_WAIT_MICROS, now.elapsed().as_micros() as f64);
}

fn register_socket(client_id: &String, socket: SocketAddr) {
    let now = Instant::now();
    trace!("Broker::register_socket");
    increment_gauge!(BROKER_SOCKET2ID_WAIT_COUNT, 1.0);
    match socket2id.insert(socket.clone(), client_id.clone()) {
        None => {
            trace!("Registered socket2id: {:?} -> {:?}", socket, client_id);
            increment_gauge!(BROKER_CONNECTED_CLIENTS_COUNT, 1.0);
        }
        Some(_) => {
            error!("Need to handle 'session taken over' case");
        }
    };
    decrement_gauge!(BROKER_SOCKET2ID_WAIT_COUNT, 1.0);

    increment_gauge!(BROKER_ID2SOCKET_WAIT_COUNT, 1.0);
    match id2socket.insert(client_id.clone(), socket.clone()) {
        None => {
            trace!("Registered id2socket: {:?} -> {:?}", client_id, socket);
        }
        Some(previous_socket) => {
            error!("Need to handle 'session taken over' case");
        }
    };
    decrement_gauge!(BROKER_ID2SOCKET_WAIT_COUNT, 1.0);
    gauge!(BROKER_REGISTER_SOCKET_WAIT_MICROS, now.elapsed().as_micros() as f64);
}

fn persist_packets(client_ids: &Vec<String>, publish_packet: &ControlPacket) {
    trace!("Broker::persist_packets");
    for client_id in client_ids {
        id2session.get_mut(client_id).unwrap()
            .register_publish(client_id.clone(), publish_packet);
    }
}

fn register_session(client_id: &String) -> SessionState {
    trace!("Broker::register_session");
    if id2session.contains_key(client_id) {
        return SessionState::SessionPresent;
    }

    return match id2session.insert(client_id.clone(), Session::new()) {
        None => {
            trace!("Created new Session for client: {:?}", client_id);
            SessionState::CleanSession
        }
        Some(session) => {
            error!("Need to handle 'session taken over' case");
            SessionState::SessionPresent
        }
    };
}

fn register_clean_session(client_id: &String) {
    trace!("Broker::register_clean_session");
    id2session.insert(client_id.clone(), Session::new());
}

async fn send_packet(client: SocketAddr, packet: ControlPacket) -> Result<(), String> {
    return send_packets(vec![client], packet).await;
}

async fn send_packets(clients: Vec<SocketAddr>, control_packet: ControlPacket) -> Result<(), String> {
    trace!("Broker::send_packets");
    let clients_count = clients.len();

    let buffer = match encode_packet(&control_packet) {
        Ok(buffer) => {
            trace!("Successfully encoded packet");
            buffer
        }
        Err(err) => {
            error!("Can't encode Control Packet: {:?}", err);
            return Err(format!("Can't encode Control Packet: {:?}", err));
        }
    };
    for socket in clients {
        if let Some(mut stream) = socket2stream.get_mut(&socket) {
            trace!("Sending packet to {:?}", socket);
            match write_buffer(&buffer, stream.borrow_mut()).await {
                Ok(_) => {
                    trace!("Successfully sent packet");
                    increment_counter!(BROKER_SENT_PACKETS_COUNT);
                }
                Err(err) => {
                    error!("Can't send data to client {:?}: {:?}", socket, err);
                    continue;
                }
            }
        } else {
            error!("Can't find client's {:?} outgoing connection", socket);
        }
    }
    trace!("Done sending packets");
    Ok(())
}


fn generate_client_id() -> String {
    trace!("Broker::generate_client_id");
    let random_prefix: u64 = rand::thread_rng().gen();
    let now = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    return random_prefix.to_string() + &now.to_string();
}