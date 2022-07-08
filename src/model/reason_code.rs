
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
