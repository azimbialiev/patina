use std::collections::{HashMap, HashSet};
use crate::mqtt::{ControlPacket, QoSLevel};
use log::{trace, info, debug, warn, error};
use metrics::{gauge, register_gauge, register_counter, increment_gauge, decrement_gauge, increment_counter};

pub static SESSION_PERSISTED_PACKETS_COUNT: &str = "session.persisted.packets.count";
pub static SESSION_PERSISTED_QOS0_PACKETS_COUNT: &str = "session.persisted.qos0.packets.count";
pub static SESSION_PERSISTED_QOS1_PACKETS_COUNT: &str = "session.persisted.qos1.packets.count";
pub static SESSION_PERSISTED_QOS2_PACKETS_COUNT: &str = "session.persisted.qos2.packets.count";
pub static SESSION_PUBACK_PACKETS_COUNT: &str = "session.puback.count";
pub static SESSION_PUBREL_PACKETS_COUNT: &str = "session.pubrel.count";
static mut METRICS_REGISTERED: bool = false;

pub enum SessionState {
    SessionPresent,
    CleanSession
}

#[derive(Debug)]
pub struct Session {
    client2pub_qos0_packets: HashMap<String, Vec<ControlPacket>>,
    client2pub_qos1_packets: HashMap<(String, u16), ControlPacket>,
    client2pub_qos2_packets: HashMap<(String, u16), ControlPacket>,
    client2puback: HashMap<(String, u16), bool>,
    client2pubrel: HashMap<(String, u16), bool>,

}

impl Session {
    pub fn register_publish(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_publish");
        increment_gauge!(SESSION_PERSISTED_PACKETS_COUNT, 1.0);
        let qos = packet.fixed_header().qos_level();
        match qos {
            QoSLevel::AtMostOnce => {
                if self.client2pub_qos0_packets.contains_key(&client_id) {
                    self.client2pub_qos0_packets
                        .entry(client_id)
                        .and_modify(|f| {
                            f.push(packet.clone());
                        });
                } else {
                    self.client2pub_qos0_packets.insert(client_id, vec![packet.clone()]);
                }
                increment_gauge!(SESSION_PERSISTED_QOS0_PACKETS_COUNT, 1.0);
            }
            QoSLevel::AtLeastOnce => {
                let packet_id = packet.variable_header().unwrap().packet_identifier().unwrap();
                self.client2pub_qos1_packets.insert((client_id, packet_id), packet.clone());
                increment_gauge!(SESSION_PERSISTED_QOS1_PACKETS_COUNT, 1.0);
            }
            QoSLevel::ExactlyOnce => {
                let packet_id = packet.variable_header().unwrap().packet_identifier().unwrap();
                self.client2pub_qos2_packets.insert((client_id, packet_id), packet.clone());
                increment_gauge!(SESSION_PERSISTED_QOS2_PACKETS_COUNT, 1.0);
            }
        }
    }
    pub fn register_puback(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_puback");
        let packet_id = packet.variable_header().unwrap().packet_identifier().unwrap();
        if self.client2puback.contains_key(&(client_id.clone(), packet_id)) {
            self.client2puback.insert((client_id, packet_id), true);
            increment_gauge!(SESSION_PUBACK_PACKETS_COUNT, 1.0);
        }
    }
    pub fn register_pubrel(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_pubrel");
        let packet_id = packet.variable_header().unwrap().packet_identifier().unwrap();
        if self.client2pubrel.contains_key(&(client_id.clone(), packet_id)) {
            self.client2pubrel.insert((client_id, packet_id), true);
            increment_gauge!(SESSION_PUBREL_PACKETS_COUNT, 1.0);
        }
    }
    pub fn is_puback_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().unwrap().packet_identifier().unwrap();
        return match self.client2puback.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }
    pub fn is_pubrel_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().unwrap().packet_identifier().unwrap();
        return match self.client2pubrel.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }
}

impl Session {
    pub fn new() -> Self {
        register_metrics();
        let client2pub_qos0_packets: HashMap<String, Vec<ControlPacket>> = HashMap::new();
        let client2pub_qos1_packets: HashMap<(String, u16), ControlPacket> = HashMap::new();
        let client2pub_qos2_packets: HashMap<(String, u16), ControlPacket> = HashMap::new();
        let client2puback: HashMap<(String, u16), bool> = HashMap::new();
        let client2pubrel: HashMap<(String, u16), bool> = HashMap::new();
        Session { client2pub_qos0_packets, client2pub_qos1_packets, client2pub_qos2_packets, client2puback, client2pubrel }
    }
}

fn register_metrics() {
    unsafe {
        if METRICS_REGISTERED {
            return;
        } else {
            METRICS_REGISTERED = true;
        }
    }
    register_gauge!(SESSION_PERSISTED_PACKETS_COUNT);
    register_gauge!(SESSION_PERSISTED_QOS0_PACKETS_COUNT);
    register_gauge!(SESSION_PERSISTED_QOS1_PACKETS_COUNT);
    register_gauge!(SESSION_PERSISTED_QOS2_PACKETS_COUNT);
    register_gauge!(SESSION_PUBACK_PACKETS_COUNT);
    register_gauge!(SESSION_PUBREL_PACKETS_COUNT);
}

