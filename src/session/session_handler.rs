use std::collections::HashMap;

use log::trace;

use crate::serdes::mqtt::{ControlPacket, QoSLevel};

pub enum SessionState {
    SessionPresent,
    CleanSession,
}

#[derive(Debug)]
pub struct SessionHandler {
    client2pub_qos0_packets: HashMap<String, Vec<ControlPacket>>,
    client2pub_qos1_packets: HashMap<(String, u16), ControlPacket>,
    client2pub_qos2_packets: HashMap<(String, u16), ControlPacket>,
    client2puback: HashMap<(String, u16), bool>,
    client2pubrel: HashMap<(String, u16), bool>,
    client2pubrec: HashMap<(String, u16), bool>,

}

impl SessionHandler {
    pub fn register_publish(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_publish");
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
            }
            QoSLevel::AtLeastOnce => {
                let packet_id = packet.variable_header().packet_identifier();
                self.client2pub_qos1_packets.insert((client_id, packet_id), packet.clone());
            }
            QoSLevel::ExactlyOnce => {
                let packet_id = packet.variable_header().packet_identifier();
                self.client2pub_qos2_packets.insert((client_id, packet_id), packet.clone());
            }
        }
    }
    pub fn register_puback(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_puback");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2puback.contains_key(&(client_id.clone(), packet_id)) {
            self.client2puback.insert((client_id, packet_id), true);
        }
    }
    pub fn register_pubrel(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_pubrel");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2pubrel.contains_key(&(client_id.clone(), packet_id)) {
            self.client2pubrel.insert((client_id, packet_id), true);
        }
    }
    pub fn register_pubrec(&mut self, client_id: String, packet: &ControlPacket) {
        trace!("Session::register_pubrec");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2pubrec.contains_key(&(client_id.clone(), packet_id)) {
            self.client2pubrec.insert((client_id, packet_id), true);
        }
    }
    pub fn is_puback_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().packet_identifier();
        return match self.client2puback.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }
    pub fn is_pubrel_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().packet_identifier();
        return match self.client2pubrel.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }
    pub fn is_pubrec_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().packet_identifier();
        return match self.client2pubrec.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }
}

impl SessionHandler {
    pub fn new() -> Self {
        let client2pub_qos0_packets: HashMap<String, Vec<ControlPacket>> = HashMap::new();
        let client2pub_qos1_packets: HashMap<(String, u16), ControlPacket> = HashMap::new();
        let client2pub_qos2_packets: HashMap<(String, u16), ControlPacket> = HashMap::new();
        let client2puback: HashMap<(String, u16), bool> = HashMap::new();
        let client2pubrel: HashMap<(String, u16), bool> = HashMap::new();
        let client2pubrec: HashMap<(String, u16), bool> = HashMap::new();

        SessionHandler { client2pub_qos0_packets, client2pub_qos1_packets, client2pub_qos2_packets, client2puback, client2pubrel, client2pubrec }
    }
}
