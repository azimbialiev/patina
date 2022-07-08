use crate::model::fixed_header::{ControlPacketType, FixedHeader};
use crate::model::payload::Payload;
use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;
use crate::model::topic::{RetainHandling, TopicFilter};
use crate::model::variable_header::{ConnectAcknowledgeFlags, ConnectFlags, Property, VariableHeader};

#[derive(Debug)]
#[derive(Clone)]
pub struct ControlPacket {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader>,
    payload: Option<Payload>,
}

impl ControlPacket {
    pub fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }
    pub fn variable_header_opt(&self) -> Option<&VariableHeader> {
        self.variable_header.as_ref()
    }

    pub fn variable_header(&self) -> &VariableHeader {
        self.variable_header.as_ref().expect("VariableHeader")
    }


    pub fn payload_opt(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    pub fn payload(&self) -> &Payload {
        self.payload.as_ref().expect("Payload")
    }

    pub fn has_client_id(&self) -> bool {
        self.payload_opt().is_some() &&
            self.payload_opt().unwrap().client_id_opt().is_some() &&
            self.payload_opt().unwrap().client_id_opt().unwrap().len() > 0
    }
}

impl ControlPacket {
    pub fn new(fixed_header: FixedHeader, variable_header: Option<VariableHeader>, payload: Option<Payload>) -> Self {
        ControlPacket { fixed_header, variable_header, payload }
    }
    pub fn connect(
        connect_flags: ConnectFlags,
        keep_alive: Option<u16>,
        properties: Vec<Property>,
        client_id: Option<String>,
        will_properties: Option<Vec<Property>>,
        will_topic: Option<String>,
        will_payload: Option<Vec<u8>>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::CONNECT, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_connect(Some(String::from("MQTT")), Some(5),
                                                           Some(connect_flags), keep_alive,
                                                           properties);
        let payload = Payload::from_connect(client_id, will_properties, will_topic, will_payload, username, password);
        let connect_packet = ControlPacket::new(fixed_header, Some(variable_header), Some(payload));
        return connect_packet;
    }
    pub fn connack(session_present: bool) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::CONNACK, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_connack(ConnectAcknowledgeFlags::new(session_present), ReasonCode::Success, vec![]);
        let connack_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return connack_packet;
    }
    pub fn subscribe(packet_identifier: Option<u16>, topic_filter: String, maximum_qos: QoSLevel) -> Self {
        let topic_filter = TopicFilter::from_subscribe(topic_filter, maximum_qos, false, false, RetainHandling::DontSendRetainedMessages, vec![]);
        let payload = Payload::from_sub_unsub(vec![topic_filter]);
        let variable_header = VariableHeader::from_sub_unsub(packet_identifier, vec![]);
        let fixed_header = FixedHeader::new(ControlPacketType::SUBSCRIBE, vec![false, false, false, false], 0);

        let subscribe_packet = ControlPacket::new(fixed_header, Some(variable_header), Some(payload));
        return subscribe_packet;
    }
    pub fn suback(packet_identifier: Option<u16>, reason_codes: Vec<ReasonCode>) -> Self {
        let payload = Payload::from_sub_unsub_ack(Option::from(reason_codes));
        let variable_header = VariableHeader::from_suback(packet_identifier, vec![]);
        let fixed_header = FixedHeader::new(ControlPacketType::SUBACK, vec![false, false, false, false], 0);

        let suback_packet = ControlPacket::new(fixed_header, Some(variable_header), Some(payload));
        return suback_packet;
    }
    pub fn unsuback(packet_identifier: Option<u16>, reason_codes: Vec<ReasonCode>) -> Self {
        let payload = Payload::from_sub_unsub_ack(Option::from(reason_codes));
        let variable_header = VariableHeader::from_suback(packet_identifier, vec![]);
        let fixed_header = FixedHeader::new(ControlPacketType::UNSUBACK, vec![false, false, false, false], 0);

        let suback_packet = ControlPacket::new(fixed_header, Some(variable_header), Some(payload));
        return suback_packet;
    }
    pub fn publish(packet_identifier: Option<u16>, topic_name: Option<String>, dup_flag: bool, qos_level: QoSLevel, retain: bool) -> Self {
        let fixed_header = FixedHeader::from_publish(dup_flag, qos_level, retain, u64::MAX);
        let variable_header = VariableHeader::from_publish(packet_identifier, topic_name, vec![]);
        let publish_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return publish_packet;
    }
    pub fn puback(packet_identifier: Option<u16>) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::PUBACK, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_pub_ack_rel_comp(packet_identifier, Some(ReasonCode::Success), vec![]);
        let puback_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return puback_packet;
    }
    pub fn pubrec(packet_identifier: Option<u16>) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::PUBREC, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_pub_ack_rel_comp(packet_identifier, Some(ReasonCode::Success), vec![]);
        let pubrec_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return pubrec_packet;
    }
    pub fn pubrel(packet_identifier: Option<u16>) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::PUBREL, vec![false, true, false, false], 0); //TODO why are they inverted?
        let variable_header = VariableHeader::from_pub_ack_rel_comp(packet_identifier, Some(ReasonCode::Success), vec![]);
        let pubrel_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return pubrel_packet;
    }
    pub fn pubcomp(packet_identifier: Option<u16>) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::PUBCOMP, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_pub_ack_rel_comp(packet_identifier, Some(ReasonCode::Success), vec![]);
        let pubcomp_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return pubcomp_packet;
    }
    pub fn pingresp() -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::PINGRESP, vec![false, false, false, false], 0);
        let pingresp_packet = ControlPacket::new(fixed_header, None, None);
        return pingresp_packet;
    }
    pub fn disconnect(reason_code: ReasonCode) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::DISCONNECT, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_disconnect(reason_code, vec![]);
        let disconnect_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return disconnect_packet;
    }
}

impl ControlPacket {}