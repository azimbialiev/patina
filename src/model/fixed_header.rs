use crate::model::qos_level::QoSLevel;

#[derive(Debug)]
#[derive(Clone)]
pub struct FixedHeader {
    packet_type: ControlPacketType,
    control_flags: Option<Vec<bool>>,
    remaining_length: u64,
    dup_flag: Option<bool>,
    qos_level: Option<QoSLevel>,
    retain: Option<bool>,
}

impl FixedHeader {
    pub fn packet_type(&self) -> ControlPacketType {
        self.packet_type
    }
    pub fn control_flags(&self) -> &Vec<bool> {
        &self.control_flags.as_ref().unwrap()
    }
    pub fn remaining_length(&self) -> u64 {
        self.remaining_length
    }
    pub fn dup_flag(&self) -> &bool {
        self.dup_flag.as_ref().unwrap()
    }

    pub fn qos_level(&self) -> &QoSLevel {
        self.qos_level.as_ref().unwrap()
    }
    pub fn retain(&self) -> &bool {
        self.retain.as_ref().unwrap()
    }
}

impl FixedHeader {
    pub fn new(packet_type: ControlPacketType, control_flags: Vec<bool>, remaining_length: u64) -> Self {
        FixedHeader {
            packet_type,
            control_flags: Some(control_flags),
            remaining_length,
            dup_flag: None,
            qos_level: None,
            retain: None,

        }
    }

    pub fn from_publish(dup_flag: bool,
                        qos_level: QoSLevel,
                        retain: bool, remaining_length: u64) -> FixedHeader {
        FixedHeader {
            packet_type: ControlPacketType::PUBLISH,
            control_flags: None,
            remaining_length,
            dup_flag: Some(dup_flag),
            qos_level: Some(qos_level),
            retain: Some(retain),
        }
    }
}


#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
pub enum ControlPacketType {
    //Reserved
    RESERVED,
    //Connection request
    CONNECT,
    //Connect acknowledgment
    CONNACK,
    //Publish message
    PUBLISH,
    //Publish acknowledgment (QoS 1)
    PUBACK,
    //Publish received (QoS 2 delivery part 1)
    PUBREC,
    //Publish release (QoS 2 delivery part 2)
    PUBREL,
    //Publish complete (QoS 2 delivery part 3)
    PUBCOMP,
    //Subscribe request
    SUBSCRIBE,
    //Subscribe acknowledgment
    SUBACK,
    //Unsubscribe request
    UNSUBSCRIBE,
    //Unsubscribe acknowledgment
    UNSUBACK,
    //PING request
    PINGREQ,
    //PING response
    PINGRESP,
    //Disconnect notification
    DISCONNECT,
    //Authentication exchange
    AUTH,
}

impl ControlPacketType {
    pub fn as_u8(&self) -> u8 {
        return match self {
            ControlPacketType::RESERVED => 0x00_u8,
            ControlPacketType::CONNECT => 0x10_u8,
            ControlPacketType::CONNACK => 0x20_u8,
            ControlPacketType::PUBLISH => 0x30_u8,
            ControlPacketType::PUBACK => 0x40_u8,
            ControlPacketType::PUBREC => 0x50_u8,
            ControlPacketType::PUBREL => 0x60_u8,
            ControlPacketType::PUBCOMP => 0x70_u8,
            ControlPacketType::SUBSCRIBE => 0x80_u8,
            ControlPacketType::SUBACK => 0x90_u8,
            ControlPacketType::UNSUBSCRIBE => 0xA0_u8,
            ControlPacketType::UNSUBACK => 0xB0_u8,
            ControlPacketType::PINGREQ => 0xC0_u8,
            ControlPacketType::PINGRESP => 0xD0_u8,
            ControlPacketType::DISCONNECT => 0xE0_u8,
            ControlPacketType::AUTH => 0xF0_u8,
        };
    }

    pub fn from_u8(value: u8) -> Option<ControlPacketType> {
        let packet_type = match value {
            0 => ControlPacketType::RESERVED,
            1 => ControlPacketType::CONNECT,
            2 => ControlPacketType::CONNACK,
            3 => ControlPacketType::PUBLISH,
            4 => ControlPacketType::PUBACK,
            5 => ControlPacketType::PUBREC,
            6 => ControlPacketType::PUBREL,
            7 => ControlPacketType::PUBCOMP,
            8 => ControlPacketType::SUBSCRIBE,
            9 => ControlPacketType::SUBACK,
            10 => ControlPacketType::UNSUBSCRIBE,
            11 => ControlPacketType::UNSUBACK,
            12 => ControlPacketType::PINGREQ,
            13 => ControlPacketType::PINGRESP,
            14 => ControlPacketType::DISCONNECT,
            15 => ControlPacketType::AUTH,
            _ => {
                return None;
            }
        };
        Some(packet_type)
    }
}


