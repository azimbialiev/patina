use crate::model::control_packet::ControlPacket;
use crate::model::qos_level::QoSLevel;
use crate::model::variable_header::ConnectFlags;

pub fn create_connect_packet(client_id: String) -> ControlPacket {
    ControlPacket::connect(
        ConnectFlags::new(false, false, false, QoSLevel::AtMostOnce, false, true, false),
        None,
        vec![],
        Some(client_id),
        None,
        None,
        None,
        None,
        None)
}

pub fn create_subscribe_packet(packet_identifier: u16, topic_filter: String, maximum_qos: QoSLevel) -> ControlPacket {
    ControlPacket::subscribe(
        Some(packet_identifier),
        topic_filter,
        maximum_qos)
}

pub fn create_publish_packet_qos0(packet_identifier: u16, topic_name: String) -> ControlPacket {
    ControlPacket::publish(
        Some(packet_identifier),
        Some(topic_name),
        false,
        QoSLevel::AtMostOnce,
        false,
    )
}

pub fn create_publish_packet_qos1(packet_identifier: u16, topic_name: String) -> ControlPacket {
    ControlPacket::publish(
        Some(packet_identifier),
        Some(topic_name),
        false,
        QoSLevel::AtLeastOnce,
        false,
    )
}