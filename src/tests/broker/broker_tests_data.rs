use std::net::{IpAddr, SocketAddr};

use log::{info, log, warn};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::init_logging;
use crate::serdes::mqtt::{ConnectFlags, ControlPacket, ControlPacketType, QoSLevel};
use crate::topic::topic_handler;

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