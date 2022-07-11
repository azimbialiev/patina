use tokio::net::tcp::OwnedReadHalf;

pub type ReadResult<T> = Result<T, ReadError>;
pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReadError {
    ConnectionError,
    NotEnoughData {
        position: u64,
        length: u64,
        requested: u64,
    },
    TooManyBitsForType {
        position: u64,
        requested: u8,
        allowed: u8,
    },
    ExceededMaxLength,
    ExceededMaxValue {
        current: u64,
        max: u64,
    },
    InvalidData,
    IOError,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DecodeError {
    VariableHeaderAndPayload { cause: ReadError },
    ConnectionTimedOut { cause: ReadError },
    VariableByteInteger { cause: ReadError },
    UTF8String { cause: ReadError },
    BinaryData { cause: ReadError },
    PacketType { cause: ReadError },
    RemainingLength { cause: ReadError },
    ProtocolName { cause: ReadError },
    ProtocolVersion { cause: ReadError },
    ConnectFlags { cause: ReadError },
    PropertyLength { cause: ReadError },
    UnknownProperty { cause: ReadError },
    KeepAlive { cause: ReadError },
    ClientId { cause: ReadError },
    Username { cause: ReadError },
    Password { cause: ReadError },
    WillProperties { cause: ReadError },
    WillTopic { cause: ReadError },
    WillPayload { cause: ReadError },
    ControlFlags { cause: ReadError },
    UsernameFlag { cause: ReadError },
    PasswordFlag { cause: ReadError },
    WillRetainFlag { cause: ReadError },
    WillQoSFlag { cause: ReadError },
    CleanStartFlag { cause: ReadError },
    WillFlag { cause: ReadError },
    ReservedFlag { cause: ReadError },
    Property { cause: ReadError },
    RetainHandling { cause: ReadError },
    MaximumQoS { cause: ReadError },
    TopicFilter { cause: ReadError },
    RetainAsPublished { cause: ReadError },
    NoLocal { cause: ReadError },
    PacketIdentifier { cause: ReadError },
    QoSLevel { cause: ReadError },
    DupFlag { cause: ReadError },
    RetainFlag { cause: ReadError },
    TopicName { cause: ReadError },
    Payload { cause: ReadError },
    ReasonCode { cause: ReadError },

}

impl DecodeError {
    pub(crate) fn cause(&self) -> ReadError {
        return match &self {
            DecodeError::VariableHeaderAndPayload { cause } => { cause.clone() }
            DecodeError::ConnectionTimedOut { cause } => { cause.clone() }
            DecodeError::VariableByteInteger { cause } => { cause.clone() }
            DecodeError::UTF8String { cause } => { cause.clone() }
            DecodeError::BinaryData { cause } => { cause.clone() }
            DecodeError::PacketType { cause } => { cause.clone() }
            DecodeError::RemainingLength { cause } => { cause.clone() }
            DecodeError::ProtocolName { cause } => { cause.clone() }
            DecodeError::ProtocolVersion { cause } => { cause.clone() }
            DecodeError::ConnectFlags { cause } => { cause.clone() }
            DecodeError::PropertyLength { cause } => { cause.clone() }
            DecodeError::UnknownProperty { cause } => { cause.clone() }
            DecodeError::KeepAlive { cause } => { cause.clone() }
            DecodeError::ClientId { cause } => { cause.clone() }
            DecodeError::Username { cause } => { cause.clone() }
            DecodeError::Password { cause } => { cause.clone() }
            DecodeError::WillProperties { cause } => { cause.clone() }
            DecodeError::WillTopic { cause } => { cause.clone() }
            DecodeError::WillPayload { cause } => { cause.clone() }
            DecodeError::ControlFlags { cause } => { cause.clone() }
            DecodeError::UsernameFlag { cause } => { cause.clone() }
            DecodeError::PasswordFlag { cause } => { cause.clone() }
            DecodeError::WillRetainFlag { cause } => { cause.clone() }
            DecodeError::WillQoSFlag { cause } => { cause.clone() }
            DecodeError::CleanStartFlag { cause } => { cause.clone() }
            DecodeError::WillFlag { cause } => { cause.clone() }
            DecodeError::ReservedFlag { cause } => { cause.clone() }
            DecodeError::Property { cause } => { cause.clone() }
            DecodeError::RetainHandling { cause } => { cause.clone() }
            DecodeError::MaximumQoS { cause } => { cause.clone() }
            DecodeError::TopicFilter { cause } => { cause.clone() }
            DecodeError::RetainAsPublished { cause } => { cause.clone() }
            DecodeError::NoLocal { cause } => { cause.clone() }
            DecodeError::PacketIdentifier { cause } => { cause.clone() }
            DecodeError::QoSLevel { cause } => { cause.clone() }
            DecodeError::DupFlag { cause } => { cause.clone() }
            DecodeError::RetainFlag { cause } => { cause.clone() }
            DecodeError::TopicName { cause } => { cause.clone() }
            DecodeError::Payload { cause } => { cause.clone() }
            DecodeError::ReasonCode { cause } => { cause.clone() }
        };
    }
}