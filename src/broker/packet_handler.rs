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
use dashmap::DashMap;
use crate::{ClientHandler, TopicHandler};

lazy_static! {

    static  ref id2session: DashMap<String, SessionHandler> = {
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


#[derive(Default, Debug)]
pub struct PacketHandlerImpl {
    pub(crate) metrics: PacketHandlerMetrics,
    client_handler: ClientHandler,
    topic_handler: TopicHandler
}

#[metered(registry = PacketHandlerMetrics)]
impl PacketHandlerImpl {
    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub(crate) async fn process_message(&self,
                                        socket: SocketAddr,
                                        control_packet: ControlPacket,
                                        to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>,
    ) -> Result<(), String> {
        debug!("Going to handle control packet: {:?} from client {:?} on socket {:?}",
        control_packet.fixed_header().packet_type(),match self.client_handler.get_client_id(&socket)
            {Err(_) => {String::from("<CLIENT_ID NOT REGISTERED>")}, Ok(client_id) => {client_id.clone()}},
            socket);

        let result = match control_packet.fixed_header().packet_type() {
            ControlPacketType::RESERVED => {}
            ControlPacketType::CONNECT => {
                let mut client_id = generate_client_id();
                if control_packet.has_client_id() {
                    debug!("Using client's client_id");
                    client_id = control_packet.payload().client_id().to_string();
                }
                info!("CONNECT client: {:?}", client_id);

                if let Some(previous_socket) = self.client_handler.register(&socket, &client_id) {
                    info!("Found a previous connection on socket {:?} for client_id {:?}", previous_socket, client_id);
                    let disconnect_packet = ControlPacket::disconnect(ReasonCode::SessionTakenOver);
                    send_packet(previous_socket, disconnect_packet, to_listener.clone()).await.expect("panic send_packet");
                }

                let mut session_present = false;
                if control_packet.variable_header().connect_flags().clean_start_flag() {
                    debug!("Creating clean session for client: {:?}", client_id);
                    register_clean_session(&client_id);
                    self.topic_handler.unsubscribe_all(&client_id);
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
                let client_id = self.client_handler.get_client_id(&socket)?;
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
                let subscribers =self.topic_handler.find_subscribers(topic_filter);
                info!("PUBLISH client: {:?} to topic:{:?}. Subscribers count: {:?}", client_id, topic_filter, subscribers.len());
                trace!("Found subscribers {:?} for topic {:?}", subscribers, topic_filter);

                persist_packets(&subscribers, &control_packet);
                let clients = subscribers
                    .iter()
                    .map(|receiver| {
                        self.client_handler.get_socket(receiver)
                    })
                    .filter(Result::is_ok)
                    .map(|c| c.unwrap().clone())
                    .filter(|receiver| { receiver.ne(&socket) })
                    .collect();
                send_packets(clients, control_packet, to_listener).await.expect("panic sending packets");
            }
            ControlPacketType::PUBACK => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                id2session.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
                // id2session.lock().await.get_mut(&client_id).unwrap().register_puback(client_id.clone(), &control_packet);
            }
            ControlPacketType::PUBREC => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                id2session.get_mut(&client_id).unwrap().register_pubrec(client_id.clone(), &control_packet);
                trace!("Sending PUBREL for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                let pubrel_packet = ControlPacket::pubrel(control_packet.variable_header().packet_identifier_opt());
                send_packet(socket, pubrel_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PUBREL => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                id2session.get_mut(&client_id).unwrap().register_pubrel(client_id.clone(), &control_packet);
                trace!("Sending PUBCOMP for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
                let pubcomp_packet = ControlPacket::pubcomp(control_packet.variable_header().packet_identifier_opt());
                send_packet(socket, pubcomp_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PUBCOMP => {}
            ControlPacketType::SUBSCRIBE => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                let topic_filters = control_packet.payload().topic_filters();
                info!("SUBSCRIBE client: {:?} to topics: {:?}", client_id, topic_filters);

                let mut reason_codes = Vec::with_capacity(topic_filters.len());
                for topic_filter in topic_filters {
                    self.topic_handler.subscribe(&client_id, topic_filter.topic_filter());
                    reason_codes.push(ReasonCode::GrantedQoS0);
                    debug!("Subscribed client {:?} to topic {:?}", client_id, topic_filter.topic_filter());
                }
                let suback_packet = ControlPacket::suback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

                send_packet(socket, suback_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::SUBACK => {}
            ControlPacketType::UNSUBSCRIBE => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                let topic_filters = control_packet.payload().topic_filters();
                info!("UNSUBSCRIBE client: {:?} from topics: {:?}", client_id, topic_filters);
                let mut reason_codes = Vec::with_capacity(topic_filters.len());
                for topic_filter in topic_filters {
                    self.topic_handler.unsubscribe(&client_id, topic_filter.topic_filter());
                    reason_codes.push(ReasonCode::Success);
                    debug!("Unsubscribed client {:?} from topic {:?}", client_id, topic_filter.topic_filter());
                }
                let unsuback_packet = ControlPacket::unsuback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

                send_packet(socket, unsuback_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::UNSUBACK => {}
            ControlPacketType::PINGREQ => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                debug!("PINGREQ from client {:?}", client_id);
                let pingresp_packet = ControlPacket::pingresp();
                send_packet(socket, pingresp_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::PINGRESP => {}
            ControlPacketType::DISCONNECT => {
                let client_id = self.client_handler.get_client_id(&socket)?;
                info!("Got a DISCONNECT packet for client {:?}. Going to clean outgoing connections", client_id);
                debug!("Disconnect reason: {:?}. Properties: {:?}", if let Some(header) = control_packet.variable_header_opt() {header.reason_code()} else {None}, if let Some(header) = control_packet.variable_header_opt() {Some(header.properties())} else {None});
                self.client_handler.unregister(&socket, &client_id);
                let disconnect_packet = ControlPacket::disconnect(ReasonCode::NormalDisconnection);
                send_packet(socket, disconnect_packet, to_listener).await.expect("panic send_packet");
            }
            ControlPacketType::AUTH => {}
        };
        Ok(())
    }
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

async fn send_packet(socket: SocketAddr, packet: ControlPacket, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Result<(), String> {
    return send_packets(vec![socket], packet, to_listener).await;
}

async fn send_packets(sockets: Vec<SocketAddr>, control_packet: ControlPacket, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Result<(), String> {
    trace!("Broker::send_packets");
    match to_listener.send((sockets, control_packet.clone())).await {
        Ok(_) => {
            trace!("Successfully sent packet");
        }
        Err(err) => {
            error!("Can't send data to sockets: {:?}",  err);
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
