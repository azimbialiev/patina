use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;

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

    pub fn from_disconnect(reason_code: ReasonCode,
                           properties: Vec<Property>) -> Self {
        VariableHeader {
            protocol_name: None,
            protocol_version: None,
            connect_flags: None,
            keep_alive: None,
            connect_acknowledge_flags: None,
            reason_code: Some(reason_code),
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
    pub fn packet_identifier_opt(&self) -> Option<u16> { self.packet_identifier.clone() }
    pub fn packet_identifier(&self) -> u16 { self.packet_identifier.unwrap() }
    pub fn topic_name(&self) -> &String { self.topic_name.as_ref().unwrap() }
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