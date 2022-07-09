use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use chrono::Local;
use serde::Serializer;
use log::{debug, error, info, trace, warn};
use metered::{*};
use rand::Rng;
use tokio::sync::mpsc::Sender;
use crate::model::control_packet::ControlPacket;
use crate::model::fixed_header::ControlPacketType;
use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;
use crate::session::session_handler::{SessionHandler, SessionState};
use crate::topic::topic_handler::TopicCommand;
use dashmap::DashMap;

lazy_static! {

    static  ref id2session: DashMap<String, SessionHandler> = {
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
}

#[derive(Default, Clone, Debug)]
pub struct PacketHandler(Arc<PacketHandlerImpl>);

impl Deref for PacketHandler {
    type Target = PacketHandlerImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl serde::Serialize for PacketHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        return self.deref().serialize(serializer);
    }
}

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct PacketHandlerImpl {
    pub(crate) metrics: PacketHandlerMetrics,

}

#[metered(registry = PacketHandlerMetrics)]
impl PacketHandlerImpl {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub(crate) async fn process_message(&self,
                                        socket: SocketAddr,
                                        control_packet: ControlPacket,
                                        to_listener: Sender<(SocketAddr, ControlPacket)>,
                                        to_topic_handler: Sender<TopicCommand>,
    ) -> Result<(), ()> {
        debug!("Going to handle control packet: {:?} from client {:?} on socket {:?}",
        control_packet.fixed_header().packet_type(),match socket2id.get(&socket) {None => {String::from("<CLIENT_ID NOT REGISTERED>")}, Some(client_id) => {client_id.clone()}}, socket);

        let result = match control_packet.fixed_header().packet_type() {
            ControlPacketType::RESERVED => {}
            ControlPacketType::CONNECT => {
                let mut client_id = generate_client_id();
                if control_packet.has_client_id() {
                    debug!("Using client's client_id");
                    client_id = control_packet.payload().client_id().to_string();
                }
                info!("CONNECT client: {:?}", client_id);
                if let Some(previous_socket) = register_socket(&client_id, socket) {
                    info!("Found a previous connection on socket {:?} for client_id {:?}", previous_socket, client_id);
                    let disconnect_packet = ControlPacket::disconnect(ReasonCode::SessionTakenOver);
                    send_packet(previous_socket, disconnect_packet, to_listener.clone()).await.expect("panic send_packet");
                }
                let mut session_present = false;
                if control_packet.variable_header().connect_flags().clean_start_flag() {
                    debug!("Creating clean session for client: {:?}", client_id);
                    register_clean_session(&client_id);
                    to_topic_handler.send(TopicCommand::UnsubscribeAll { client_id }).await.expect("panic send command to topic handler");
                } else {
                    session_present = match register_session(&client_id) {
                        SessionState::SessionPresent => true,
                        SessionState::CleanSession => false
                    };
                }
                let connack_packet = ControlPacket::connack(session_present);
                send_packet(socket, connack_packet, to_listener).await.expect("panic send_packet");
                //TODO Check Auth
                //TODO Check previous session using client_id
                //TODO Check clean_start
            }
            ControlPacketType::CONNACK => {}
            ControlPacketType::PUBLISH => {
                let client_id = get_client_id(&socket).await;
                if control_packet.fixed_header().qos_level() == &QoSLevel::AtLeastOnce {
                    trace!("Sending PUBACK for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                    let puback_packet = ControlPacket::puback(control_packet.variable_header().packet_identifier_opt());
                    send_packet(socket, puback_packet, to_listener.clone()).await.expect("panic send_packet");
                } else if control_packet.fixed_header().qos_level() == &QoSLevel::ExactlyOnce {
                    trace!("Sending PUBREC for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                    let pubrec_packet = ControlPacket::pubrec(control_packet.variable_header().packet_identifier_opt());
                    send_packet(socket, pubrec_packet, to_listener.clone()).await.expect("panic send_packet");
                }
                let topic_filter = control_packet.variable_header().topic_name();
                let (callback, res) = tokio::sync::oneshot::channel();
                to_topic_handler.send(TopicCommand::FindSubscribers { topic_filter: topic_filter.clone(), callback }).await.expect("panic sending topic command");
                let subscribers = res.await.expect("panic finding subscribers");
                info!("PUBLISH client: {:?} to topic:{:?}. Subscribers count: {:?}", client_id, topic_filter, subscribers.len());
                trace!("Found subscribers {:?} for topic {:?}", subscribers, topic_filter);

                persist_packets(&subscribers, &control_packet);
                let clients = subscribers
                    .iter()
                    .map(|receiver| {
                        id2socket.get(receiver)
                    })
                    .filter(Option::is_some)
                    .map(|c| c.unwrap().clone())
                    .filter(|receiver| { receiver.ne(&socket) })
                    .collect();
                send_packets(clients, control_packet, to_listener).await.expect("panic sending packets");
            }
            ControlPacketType::PUBACK => {
                let client_id = get_client_id(&socket).await;
                id2session.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
                // id2session.lock().await.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
            }
            ControlPacketType::PUBREC => {
                let client_id = get_client_id(&socket).await;
                id2session.get_mut(&client_id).unwrap().register_pubrec(client_id.clone(), &control_packet);
                trace!("Sending PUBREL for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                let pubrel_packet = ControlPacket::pubrel(control_packet.variable_header().packet_identifier_opt());
                send_packet(socket, pubrel_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PUBREL => {
                let client_id = get_client_id(&socket).await;
                id2session.get_mut(&client_id).unwrap().register_pubrel(client_id.clone(), &control_packet);
                trace!("Sending PUBCOMP for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                let pubcomp_packet = ControlPacket::pubcomp(control_packet.variable_header().packet_identifier_opt());
                send_packet(socket, pubcomp_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PUBCOMP => {}
            ControlPacketType::SUBSCRIBE => {
                let client_id = get_client_id(&socket).await;
                let topic_filters = control_packet.payload().topic_filters();
                info!("SUBSCRIBE client: {:?} to topics: {:?}", client_id, topic_filters);

                let mut reason_codes = Vec::with_capacity(topic_filters.len());
                for topic_filter in topic_filters {
                    to_topic_handler.send(TopicCommand::Subscribe { client_id: client_id.clone(), topic_filter: topic_filter.topic_filter().clone() }).await.expect("panic adding subscribers");
                    reason_codes.push(ReasonCode::GrantedQoS0);
                    debug!("Subscribed client {:?} to topic {:?}", client_id, topic_filter.topic_filter());
                }
                let suback_packet = ControlPacket::suback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

                send_packet(socket, suback_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::SUBACK => {}
            ControlPacketType::UNSUBSCRIBE => {
                let client_id = get_client_id(&socket).await;
                let topic_filters = control_packet.payload().topic_filters();
                info!("UNSUBSCRIBE client: {:?} from topics: {:?}", client_id, topic_filters);
                let mut reason_codes = Vec::with_capacity(topic_filters.len());
                for topic_filter in topic_filters {
                    to_topic_handler.send(TopicCommand::Unsubscribe { client_id: client_id.clone(), topic_filter: topic_filter.topic_filter().clone() }).await.expect("panic adding subscribers");
                    reason_codes.push(ReasonCode::Success);
                    debug!("Unsubscribed client {:?} from topic {:?}", client_id, topic_filter.topic_filter());
                }
                let unsuback_packet = ControlPacket::unsuback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

                send_packet(socket, unsuback_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::UNSUBACK => {}
            ControlPacketType::PINGREQ => {
                let client_id = get_client_id(&socket).await;
                debug!("PINGREQ from client {:?}", client_id);
                let pingresp_packet = ControlPacket::pingresp();
                send_packet(socket, pingresp_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PINGRESP => {}
            ControlPacketType::DISCONNECT => {
                let client_id = get_client_id(&socket).await;
                info!("Got a DISCONNECT packet for client {:?}. Going to clean outgoing connections", client_id);
                debug!("Disconnect reason: {:?}. Properties: {:?}", if let Some(header) = control_packet.variable_header_opt() {header.reason_code()} else {None}, if let Some(header) = control_packet.variable_header_opt() {Some(header.properties())} else {None});
                unregister_socket(&client_id, &socket);
                let disconnect_packet = ControlPacket::disconnect(ReasonCode::NormalDisconnection);
                send_packet(socket, disconnect_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::AUTH => {}
        };
        Ok(())
    }
}

async fn get_client_id(socket: &SocketAddr) -> String {
    trace!("Broker::get_client_id");
    match socket2id.get(socket) {
        None => {
            panic!("Can't get client_id for socket {}", socket);
        }
        Some(client_id) => {
            return client_id.clone();
        }
    };
}

fn unregister_socket(client_id: &String, socket: &SocketAddr) {
    trace!("Broker::unregister_socket");
    match socket2id.remove(socket) {
        None => {
            trace!("Unregister socket2id: {:?} -> {:?}", socket, client_id);
        }
        Some(_) => {
            error!("Need to handle 'session taken over' case");
        }
    };

    match id2socket.remove(client_id) {
        None => {
            trace!("Unregister id2socket: {:?} -> {:?}", client_id, socket);
        }
        Some(previous_socket) => {
            error!("Need to handle 'session taken over' case");
        }
    };
}

fn register_socket(client_id: &String, socket: SocketAddr) -> Option<SocketAddr> {
    trace!("Broker::register_socket");
    if socket2id.contains_key(&socket) {
        warn!("The socket {} is already registered with client_id {}. New client_id: {}",client_id, socket2id.get(&socket).unwrap().to_string(), socket);
    }
    match socket2id.insert(socket.clone(), client_id.clone()) {
        None => {
            trace!("Registered socket2id: {:?} -> {:?}", socket, client_id);
        }
        Some(_) => {
            error!("Need to handle 'session taken over' case");
        }
    };

    let previous_socket = match id2socket.insert(client_id.clone(), socket.clone()) {
        None => {
            trace!("Registered id2socket: {:?} -> {:?}", client_id, socket);
            None
        }
        Some(previous_socket) => {
            info!("Found a previous socket {:?} associated to client {:?}", previous_socket, client_id);
            Some(previous_socket)
        }
    };
    return previous_socket;
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

    return match id2session.insert(client_id.clone(), SessionHandler::new()) {
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
    id2session.insert(client_id.clone(), SessionHandler::new());
}

async fn send_packet(client: SocketAddr, packet: ControlPacket, to_listener: Sender<(SocketAddr, ControlPacket)>) -> Result<(), String> {
    return send_packets(vec![client], packet, to_listener).await;
}

async fn send_packets(clients: Vec<SocketAddr>, control_packet: ControlPacket, to_listener: Sender<(SocketAddr, ControlPacket)>) -> Result<(), String> {
    trace!("Broker::send_packets");
    for socket in clients {
        match to_listener.send((socket, control_packet.clone())).await {
            Ok(_) => {
                trace!("Successfully sent packet");
            }
            Err(err) => {
                error!("Can't send data to client {:?}: {:?}", socket, err);
                continue;
            }
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
