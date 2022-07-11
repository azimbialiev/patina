use dashmap::DashMap;
use log::trace;

use crate::model::control_packet::ControlPacket;
use crate::model::qos_level::QoSLevel;
use metered::{*};

pub enum SessionState {
    SessionPresent,
    CleanSession,
}

#[derive(Debug)]
pub struct SessionHandler {
    client2pub_qos0_packets: DashMap<String, Vec<ControlPacket>>,
    client2pub_qos1_packets: DashMap<(String, u16), ControlPacket>,
    client2pub_qos2_packets: DashMap<(String, u16), ControlPacket>,
    client2puback: DashMap<(String, u16), bool>,
    client2pubrel: DashMap<(String, u16), bool>,
    client2pubrec: DashMap<(String, u16), bool>,
    pub(crate) metrics: SessionHandlerMetrics,

}

#[metered(registry = SessionHandlerMetrics)]
impl SessionHandler {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn register_publish(&self, client_id: String, packet: &ControlPacket) {
        trace!("register_publish");
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

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn register_puback(&self, client_id: String, packet: &ControlPacket) {
        trace!("register_puback");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2puback.contains_key(&(client_id.clone(), packet_id)) {
            self.client2puback.insert((client_id, packet_id), true);
        }
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn register_pubrel(&self, client_id: String, packet: &ControlPacket) {
        trace!("register_pubrel");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2pubrel.contains_key(&(client_id.clone(), packet_id)) {
            self.client2pubrel.insert((client_id, packet_id), true);
        }
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn register_pubrec(&self, client_id: String, packet: &ControlPacket) {
        trace!("register_pubrec");
        let packet_id = packet.variable_header().packet_identifier();
        if self.client2pubrec.contains_key(&(client_id.clone(), packet_id)) {
            self.client2pubrec.insert((client_id, packet_id), true);
        }
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn is_puback_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().packet_identifier();
        return match self.client2puback.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn is_pubrel_complete(&self, client_id: String, packet: &ControlPacket) -> bool {
        let packet_id = &packet.variable_header().packet_identifier();
        return match self.client2pubrel.get(&(client_id, *packet_id)) {
            None => {
                false
            }
            Some(result) => { *result }
        };
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
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
        let client2pub_qos0_packets: DashMap<String, Vec<ControlPacket>> = DashMap::new();
        let client2pub_qos1_packets: DashMap<(String, u16), ControlPacket> = DashMap::new();
        let client2pub_qos2_packets: DashMap<(String, u16), ControlPacket> = DashMap::new();
        let client2puback: DashMap<(String, u16), bool> = DashMap::new();
        let client2pubrel: DashMap<(String, u16), bool> = DashMap::new();
        let client2pubrec: DashMap<(String, u16), bool> = DashMap::new();

        SessionHandler { client2pub_qos0_packets, client2pub_qos1_packets, client2pub_qos2_packets, client2puback, client2pubrel, client2pubrec, metrics: SessionHandlerMetrics::default() }
    }
}
