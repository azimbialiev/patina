use std::fmt::Debug;
use log::{debug, error, trace, warn};

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

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
pub enum ReasonCode {
    Success,
    NormalDisconnection,
    GrantedQoS0,
    GrantedQoS1,
    GrantedQoS2,
    DisconnectWithWillMessage,
    NoMatchingSubscribers,
    NoSubscriptionExisted,
    ContinueAuthentication,
    ReAuthenticate,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUsernameOrPassword,
    NotAuthorized,
    ServerUnavailable,
    ServerBusy,
    Banned,
    ServerShuttingDown,
    BadAuthenticationMethod,
    KeepAliveTimeout,
    SessionTakenOver,
    TopicFilterInvalid,
    TopicNameInvalid,
    PacketIdentifierInUse,
    PacketIdentifierNotFound,
    ReceiveMaximumExceeded,
    TopicAliasInvalid,
    PacketTooLarge,
    MessageRateTooHigh,
    QuotaExceeded,
    AdministrativeAction,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    SharedSubscriptionsNotSupported,
    ConnectionRateExceeded,
    MaximumConnectTime,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,

}

impl ReasonCode {
    pub fn as_u8(&self) -> u8 {
        return match self {
            ReasonCode::Success => { 0x00_u8 }
            ReasonCode::NormalDisconnection => { 0x00_u8 }
            ReasonCode::GrantedQoS0 => { 0x00_u8 }
            ReasonCode::GrantedQoS1 => { 0x01_u8 }
            ReasonCode::GrantedQoS2 => { 0x02_u8 }
            ReasonCode::DisconnectWithWillMessage => { 0x04_u8 }
            ReasonCode::NoMatchingSubscribers => { 0x10_u8 }
            ReasonCode::NoSubscriptionExisted => { 0x11_u8 }
            ReasonCode::ContinueAuthentication => { 0x18_u8 }
            ReasonCode::ReAuthenticate => { 0x19_u8 }
            ReasonCode::UnspecifiedError => { 0x80_u8 }
            ReasonCode::MalformedPacket => { 0x81_u8 }
            ReasonCode::ProtocolError => { 0x82_u8 }
            ReasonCode::ImplementationSpecificError => { 0x83_u8 }
            ReasonCode::UnsupportedProtocolVersion => { 0x84_u8 }
            ReasonCode::ClientIdentifierNotValid => { 0x85_u8 }
            ReasonCode::BadUsernameOrPassword => { 0x86_u8 }
            ReasonCode::NotAuthorized => { 0x87_u8 }
            ReasonCode::ServerUnavailable => { 0x88_u8 }
            ReasonCode::ServerBusy => { 0x89_u8 }
            ReasonCode::Banned => { 0x8A_u8 }
            ReasonCode::ServerShuttingDown => { 0x8B_u8 }
            ReasonCode::BadAuthenticationMethod => { 0x8C_u8 }
            ReasonCode::KeepAliveTimeout => { 0x8D_u8 }
            ReasonCode::SessionTakenOver => { 0x8E_u8 }
            ReasonCode::TopicFilterInvalid => { 0x8F_u8 }
            ReasonCode::TopicNameInvalid => { 0x90_u8 }
            ReasonCode::PacketIdentifierInUse => { 0x91_u8 }
            ReasonCode::PacketIdentifierNotFound => { 0x92_u8 }
            ReasonCode::ReceiveMaximumExceeded => { 0x93_u8 }
            ReasonCode::TopicAliasInvalid => { 0x94_u8 }
            ReasonCode::PacketTooLarge => { 0x95_u8 }
            ReasonCode::MessageRateTooHigh => { 0x96_u8 }
            ReasonCode::QuotaExceeded => { 0x97_u8 }
            ReasonCode::AdministrativeAction => { 0x98_u8 }
            ReasonCode::PayloadFormatInvalid => { 0x99_u8 }
            ReasonCode::RetainNotSupported => { 0x9A_u8 }
            ReasonCode::QoSNotSupported => { 0x9B_u8 }
            ReasonCode::UseAnotherServer => { 0x9C_u8 }
            ReasonCode::ServerMoved => { 0x9D_u8 }
            ReasonCode::SharedSubscriptionsNotSupported => { 0x9E_u8 }
            ReasonCode::ConnectionRateExceeded => { 0x9F_u8 }
            ReasonCode::MaximumConnectTime => { 0xA0_u8 }
            ReasonCode::SubscriptionIdentifiersNotSupported => { 0xA1_u8 }
            ReasonCode::WildcardSubscriptionsNotSupported => { 0xA2_u8 }
        };
    }
    pub fn from_u8(value: u8) -> Option<ReasonCode> {
        return Some(match value {
            0x00_u8 => { ReasonCode::Success }
            0x00_u8 => { ReasonCode::NormalDisconnection }
            0x00_u8 => { ReasonCode::GrantedQoS0 }
            0x01_u8 => { ReasonCode::GrantedQoS1 }
            0x02_u8 => { ReasonCode::GrantedQoS2 }
            0x04_u8 => { ReasonCode::DisconnectWithWillMessage }
            0x10_u8 => { ReasonCode::NoMatchingSubscribers }
            0x11_u8 => { ReasonCode::NoSubscriptionExisted }
            0x18_u8 => { ReasonCode::ContinueAuthentication }
            0x19_u8 => { ReasonCode::ReAuthenticate }
            0x80_u8 => { ReasonCode::UnspecifiedError }
            0x81_u8 => { ReasonCode::MalformedPacket }
            0x82_u8 => { ReasonCode::ProtocolError }
            0x83_u8 => { ReasonCode::ImplementationSpecificError }
            0x84_u8 => { ReasonCode::UnsupportedProtocolVersion }
            0x85_u8 => { ReasonCode::ClientIdentifierNotValid }
            0x86_u8 => { ReasonCode::BadUsernameOrPassword }
            0x87_u8 => { ReasonCode::NotAuthorized }
            0x88_u8 => { ReasonCode::ServerUnavailable }
            0x89_u8 => { ReasonCode::ServerBusy }
            0x8A_u8 => { ReasonCode::Banned }
            0x8B_u8 => { ReasonCode::ServerShuttingDown }
            0x8C_u8 => { ReasonCode::BadAuthenticationMethod }
            0x8D_u8 => { ReasonCode::KeepAliveTimeout }
            0x8E_u8 => { ReasonCode::SessionTakenOver }
            0x8F_u8 => { ReasonCode::TopicFilterInvalid }
            0x90_u8 => { ReasonCode::TopicNameInvalid }
            0x91_u8 => { ReasonCode::PacketIdentifierInUse }
            0x92_u8 => { ReasonCode::PacketIdentifierNotFound }
            0x93_u8 => { ReasonCode::ReceiveMaximumExceeded }
            0x94_u8 => { ReasonCode::TopicAliasInvalid }
            0x95_u8 => { ReasonCode::PacketTooLarge }
            0x96_u8 => { ReasonCode::MessageRateTooHigh }
            0x97_u8 => { ReasonCode::QuotaExceeded }
            0x98_u8 => { ReasonCode::AdministrativeAction }
            0x99_u8 => { ReasonCode::PayloadFormatInvalid }
            0x9A_u8 => { ReasonCode::RetainNotSupported }
            0x9B_u8 => { ReasonCode::QoSNotSupported }
            0x9C_u8 => { ReasonCode::UseAnotherServer }
            0x9D_u8 => { ReasonCode::ServerMoved }
            0x9E_u8 => { ReasonCode::SharedSubscriptionsNotSupported }
            0x9F_u8 => { ReasonCode::ConnectionRateExceeded }
            0xA0_u8 => { ReasonCode::MaximumConnectTime }
            0xA1_u8 => { ReasonCode::SubscriptionIdentifiersNotSupported }
            0xA2_u8 => { ReasonCode::WildcardSubscriptionsNotSupported }
            _ => {
                return None;
            }
        });
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
pub enum QoSLevel {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl QoSLevel {
    pub fn from_u8(value: u8) -> Option<QoSLevel> {
        return match value {
            0 => {
                Option::from(QoSLevel::AtMostOnce)
            }
            1 => {
                Option::from(QoSLevel::AtLeastOnce)
            }
            2 => {
                Option::from(QoSLevel::ExactlyOnce)
            }
            _ => {
                None
            }
        };
    }

    pub fn to_bool(&self) -> (bool, bool) {
        return match self {
            QoSLevel::AtMostOnce => { (false, false) }
            QoSLevel::AtLeastOnce => { (false, true) }
            QoSLevel::ExactlyOnce => { (true, false) }
        };
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct ConnectFlags {
    username_flag: bool,
    password_flag: bool,
    will_retain_flag: bool,
    will_qos: QoSLevel,
    will_flag: bool,
    clean_start_flag: bool,
    reserved_flag: bool,
}

impl ConnectFlags {
    pub fn new(username_flag: bool, password_flag: bool, will_retain_flag: bool, will_qos: QoSLevel, will_flag: bool, clean_start_flag: bool, reserved_flag: bool) -> Self {
        ConnectFlags { username_flag, password_flag, will_retain_flag, will_qos, will_flag, clean_start_flag, reserved_flag }
    }
}

impl ConnectFlags {
    pub fn username_flag(&self) -> bool {
        self.username_flag
    }
    pub fn password_flag(&self) -> bool {
        self.password_flag
    }
    pub fn will_retain_flag(&self) -> bool {
        self.will_retain_flag
    }
    pub fn will_qos(&self) -> QoSLevel {
        self.will_qos
    }
    pub fn will_flag(&self) -> bool {
        self.will_flag
    }
    pub fn clean_start_flag(&self) -> bool {
        self.clean_start_flag
    }
    pub fn reserved_flag(&self) -> bool {
        self.reserved_flag
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct ConnectAcknowledgeFlags {
    session_present: bool,
}

impl ConnectAcknowledgeFlags {
    pub fn new(session_present: bool) -> Self {
        ConnectAcknowledgeFlags { session_present }
    }
}

impl ConnectAcknowledgeFlags {
    pub fn session_present(&self) -> bool {
        self.session_present
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct VariableHeader {
    // START CONNECT
    protocol_name: Option<String>,
    protocol_version: Option<u8>,
    connect_flags: Option<ConnectFlags>,
    keep_alive: Option<u16>,
    //END CONNECT
    //START CONNACK
    connect_acknowledge_flags: Option<ConnectAcknowledgeFlags>,
    reason_code: Option<ReasonCode>,
    //END CONNECK
    properties: Vec<Property>,
    packet_identifier: Option<u16>,
    topic_name: Option<String>,
}


impl VariableHeader {
    pub fn from_connect(protocol_name: Option<String>, protocol_version: Option<u8>,
                        connect_flags: Option<ConnectFlags>, keep_alive: Option<u16>,
                        properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name,
            protocol_version,
            connect_flags,
            keep_alive,
            connect_acknowledge_flags: None,
            reason_code: None,
            properties,
            packet_identifier: None,
            topic_name: None,
        }
    }

    pub fn from_connack(connect_acknowledge_flags: ConnectAcknowledgeFlags, connect_reason_code: ReasonCode,
                        properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: Some(connect_acknowledge_flags),
            reason_code: Some(connect_reason_code),
            properties,
            packet_identifier: None,
            topic_name: None,
        }
    }

    pub fn from_sub_unsub(packet_identifier: Option<u16>, properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: None,
            reason_code: None,
            properties,
            packet_identifier,
            topic_name: None,
        }
    }

    pub fn from_suback(packet_identifier: Option<u16>, properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: None,
            reason_code: None,
            properties,
            packet_identifier,
            topic_name: None,
        }
    }

    pub fn from_publish(packet_identifier: Option<u16>, topic_name: Option<String>, properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: None,
            reason_code: None,
            properties,
            packet_identifier,
            topic_name,
        }
    }

    pub fn from_pub_ack_rel_comp(packet_identifier: Option<u16>, reason_code: Option<ReasonCode>, properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: None,
            reason_code,
            properties,
            packet_identifier,
            topic_name: None,
        }
    }
}


impl VariableHeader {
    pub fn protocol_name(&self) -> &String {
        self.protocol_name.as_ref().unwrap()
    }
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version.unwrap()
    }
    pub fn connect_flags(&self) -> &ConnectFlags {
        return self.connect_flags.as_ref().unwrap();
    }
    pub fn keep_alive(&self) -> u16 {
        self.keep_alive.unwrap()
    }

    pub fn connect_acknowledge_flags(&self) -> &ConnectAcknowledgeFlags {
        self.connect_acknowledge_flags.as_ref().unwrap()
    }

    pub fn reason_code(&self) -> Option<&ReasonCode> {
        self.reason_code.as_ref()
    }

    pub fn properties(&self) -> &Vec<Property> {
        self.properties.as_ref()
    }
    pub fn packet_identifier(&self) -> Option<u16> { self.packet_identifier.clone() }
    pub fn topic_name(&self) -> &String { self.topic_name.as_ref().unwrap() }
}


#[derive(Debug)]
#[derive(Clone)]
#[derive(Eq, PartialEq)]
pub enum Property {
    PayloadFormatIndicator(u8),
    MessageExpiryInterval(u32),
    ContentType(String),
    ResponseTopic(String),
    CorrelationData(Vec<u8>),
    SubscriptionIdentifier(u64),
    SessionExpiryInterval(u32),
    AssignedClientIdentifier(String),
    ServerKeepAlive(u8),
    AuthenticationMethod(String),
    AuthenticationData(Vec<u8>),
    RequestProblemInformation(u8),
    WillDelayInterval(u32),
    RequestResponseInformation(u8),
    ResponseInformation(String),
    ServerReference(String),
    ReasonString(String),
    ReceiveMaximum(u16),
    TopicAliasMaximum(u16),
    TopicAlias(u16),
    MaximumQoS(u8),
    RetainAvailable(u8),
    UserProperty(String, String),
    MaximumPacketSize(u32),
    WildcardSubscriptionAvailable(u8),
    SubscriptionIdentifierAvailable(u8),
    SharedSubscriptionAvailable(u8),
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Payload {
    client_id: Option<String>,
    will_properties: Option<Vec<Property>>,
    will_topic: Option<String>,
    will_payload: Option<Vec<u8>>,
    username: Option<String>,
    password: Option<String>,
    topic_filters: Option<Vec<TopicFilter>>,
    reason_codes: Option<Vec<ReasonCode>>,
    data: Option<Vec<u8>>,
}

impl Payload {
    pub fn client_id(&self) -> Option<&String> {
        self.client_id.as_ref()
    }
    pub fn topic_filters(&self) -> &Vec<TopicFilter> {
        self.topic_filters.as_ref().unwrap()
    }
    pub fn reason_codes(&self) -> &Vec<ReasonCode> {
        self.reason_codes.as_ref().unwrap()
    }
    pub fn data(&self) -> &Vec<u8> {
        self.data.as_ref().unwrap()
    }
}

impl Payload {
    pub fn from_connect(client_id: Option<String>, will_properties: Option<Vec<Property>>, will_topic: Option<String>, will_payload: Option<Vec<u8>>, username: Option<String>, password: Option<String>) -> Self {
        Payload {
            client_id,
            will_properties,
            will_topic,
            will_payload,
            username,
            password,
            topic_filters: None,
            reason_codes: None,
            data: None,
        }
    }

    pub fn from_sub_unsub(topic_filters: Vec<TopicFilter>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: Some(topic_filters),
            reason_codes: None,
            data: None,
        }
    }
    pub fn from_subsack(reason_codes: Option<Vec<ReasonCode>>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: None,
            reason_codes,
            data: None,
        }
    }

    pub fn from_publish(data: Option<Vec<u8>>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: None,
            reason_codes: None,
            data,
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Eq, PartialEq)]
pub enum RetainHandling {
    SendRetainedMessagesOnSubscribe,
    SendRetainedMessagesOnNewSubscribe,
    DontSendRetainedMessages,
}

impl RetainHandling {
    pub fn from_u8(value: u8) -> Option<RetainHandling> {
        let retain_handling = match value {
            0 => { RetainHandling::SendRetainedMessagesOnSubscribe }
            1 => { RetainHandling::SendRetainedMessagesOnNewSubscribe }
            2 => { RetainHandling::DontSendRetainedMessages }
            _ => { return None; }
        };
        Some(retain_handling)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TopicFilter {
    topic_filter: String,
    maximum_qos: QoSLevel,
    no_local: bool,
    retain_as_published: bool,
    retain_handling: RetainHandling,
    reserved_bits: Vec<bool>,
}

impl TopicFilter {
    pub fn from_subscribe(topic_filter: String, maximum_qos: QoSLevel, no_local: bool, retain_as_published: bool, retain_handling: RetainHandling, reserved_bits: Vec<bool>) -> Self {
        TopicFilter { topic_filter, maximum_qos, no_local, retain_as_published, retain_handling, reserved_bits }
    }
    pub fn from_unsubscribe(topic_filter: String) -> Self {
        TopicFilter { topic_filter, maximum_qos: QoSLevel::ExactlyOnce, no_local: true, retain_as_published: false, retain_handling: RetainHandling::DontSendRetainedMessages, reserved_bits: vec![] }
    }
    pub fn topic_filter(&self) -> &String {
        return &self.topic_filter;
    }
}


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
    pub fn variable_header(&self) -> Option<&VariableHeader> {
        self.variable_header.as_ref()
    }

    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    pub fn has_client_id(&self) -> bool {
        self.payload().is_some() &&
            self.payload().unwrap().client_id().is_some() &&
            self.payload().unwrap().client_id().unwrap().len() > 0
    }
}

impl ControlPacket {
    pub fn new(fixed_header: FixedHeader, variable_header: Option<VariableHeader>, payload: Option<Payload>) -> Self {
        ControlPacket { fixed_header, variable_header, payload }
    }
    pub fn connack(session_present: bool) -> Self {
        let fixed_header = FixedHeader::new(ControlPacketType::CONNACK, vec![false, false, false, false], 0);
        let variable_header = VariableHeader::from_connack(ConnectAcknowledgeFlags::new(session_present), ReasonCode::Success, vec![]);
        let connack_packet = ControlPacket::new(fixed_header, Some(variable_header), None);
        return connack_packet;
    }
    pub fn suback(packet_identifier: Option<u16>,reason_codes: Vec<ReasonCode>) -> Self {
        let payload = Payload::from_subsack(Option::from(reason_codes));
        let variable_header = VariableHeader::from_suback(packet_identifier, vec![]);
        let fixed_header = FixedHeader::new(ControlPacketType::SUBACK, vec![false, false, false, false], 0);

        let suback_packet = ControlPacket::new(fixed_header, Some(variable_header), Some(payload));
        return suback_packet;
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
}

impl ControlPacket {}
