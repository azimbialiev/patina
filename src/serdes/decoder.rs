use core::result;
use std::io::ErrorKind;

use bitreader::{BitReader, BitReaderError};
use bytes::BufMut;
use log::{debug, error, trace};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::MutexGuard;

use crate::serdes::mqtt::{ConnectFlags, ControlPacketType, FixedHeader, Payload, Property, QoSLevel, ReasonCode, RetainHandling, TopicFilter, VariableHeader};

pub type ReadResult<T> = result::Result<T, ReadError>;
pub type DecodeResult<T> = result::Result<T, DecodeError>;

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

pub trait Decoder<T> {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<T>;
    fn map_error(&self, err: BitReaderError) -> ReadError {
        return match err {
            BitReaderError::NotEnoughData {
                position,
                length,
                requested
            } => {
                ReadError::NotEnoughData { position, length, requested }
            }
            BitReaderError::TooManyBitsForType {
                position,
                requested,
                allowed
            } => {
                ReadError::TooManyBitsForType { position, requested, allowed }
            }
        };
    }

    fn read_u8(&self, bit_count: u8, reader: &mut BitReader) -> ReadResult<u8> {
        return match reader.read_u8(bit_count) {
            Ok(result) => {
                Ok(result)
            }
            Err(err) => {
                error!("Can't read {:?} bits as u8. {:?}", bit_count, err);
                return Err(self.map_error(err));
            }
        };
    }

    fn read_u16(&self, bit_count: u8, reader: &mut BitReader) -> ReadResult<u16> {
        return match reader.read_u16(bit_count) {
            Ok(result) => {
                Ok(result)
            }
            Err(err) => {
                error!("Can't read {:?} bits as u16. {:?}", bit_count, err);
                return Err(self.map_error(err));
            }
        };
    }

    fn read_u32(&self, bit_count: u8, reader: &mut BitReader) -> ReadResult<u32> {
        return match reader.read_u32(bit_count) {
            Ok(result) => {
                Ok(result)
            }
            Err(err) => {
                error!("Can't read {:?} bits as u32. {:?}", bit_count, err);
                return Err(self.map_error(err));
            }
        };
    }

    fn read_booleans(&self, bit_count: u8, reader: &mut BitReader) -> ReadResult<Vec<bool>> {
        trace!("Decoder::read_booleans");
        let mut result: Vec<bool> = Vec::with_capacity(bit_count as usize);
        for _ in 0..bit_count {
            let bool = self.read_bool(reader)?;
            result.push(bool);
        }
        return Ok(result);
    }

    fn read_bool(&self, reader: &mut BitReader) -> ReadResult<bool> {
        trace!("Decoder::read_bool");
        let result = match reader.read_bool() {
            Ok(result) => { result }
            Err(err) => {
                return Err(self.map_error(err));
            }
        };
        return Ok(result);
    }


    fn read_variable_byte_integer(&self, reader: &mut BitReader) -> DecodeResult<u64> {
        let mut multiplier: u64 = 1;
        let mut result: u64 = 0;
        let start = reader.position();
        loop {
            let encoded_byte = match self.read_u8(8, reader) {
                Ok(res) => { res }
                Err(err) => {
                    error!("Can't decode Variable Byte Integer: {:?}", err);
                    return Err(DecodeError::VariableByteInteger { cause: err });
                }
            };
            // trace!("Encoded byte: {:?}", encoded_byte);
            result += (encoded_byte & (127 as u8)) as u64 * multiplier;
            if multiplier > 128 * 128 * 128 {
                error!("Can't decode Variable Byte Integer. Multiplier: {:?} ", multiplier);
                return Err(DecodeError::VariableByteInteger { cause: ReadError::ExceededMaxValue { current: multiplier, max: 128 * 128 * 128 } });
            }
            multiplier *= 128;
            // trace!("Multiplier: {:?}", multiplier);
            //trace!("Encoded byte & 128: {:?}", encoded_byte & 128);
            if (encoded_byte & 128) == 0 {
                break;
            }
        }
        trace!("Variable Byte Integer Length: {:?}", (reader.position() - start) / 8);
        return Ok(result);
    }

    fn read_utf8_string(&self, reader: &mut BitReader) -> DecodeResult<String> {
        let string_length = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read UTF8 String length: {:?}", err);
                return Err(DecodeError::UTF8String { cause: err });
            }
        };
        trace!("Extracted UTF-8 Encoded String Length: {:?}", string_length);
        let mut value: Vec<u8> = Vec::new();
        for i in 0..string_length as usize {
            let char = match self.read_u8(8, reader) {
                Ok(result) => { result }
                Err(err) => {
                    error!("Can't read UTF8 byte: {:?}", err);
                    return Err(DecodeError::UTF8String { cause: err });
                }
            };
            value.insert(i, char);
        }
        let result = match String::from_utf8(value) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't convert bytes to UTF8 String: {:?}", err);
                return Err(DecodeError::UTF8String { cause: ReadError::InvalidData });
            }
        };
        return Ok(result);
    }

    fn read_binary_data(&self, reader: &mut BitReader) -> DecodeResult<Vec<u8>> {
        let binary_data_length = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result as usize }
            Err(err) => {
                error!("Can't read Binary Data length: {:?}", err);
                return Err(DecodeError::BinaryData { cause: err });
            }
        };
        trace!("Extracted Binary Data Length: {:?}", binary_data_length);
        let mut binary_data = Vec::with_capacity(binary_data_length);
        for _ in 0..binary_data_length {
            let byte = match self.read_u8(8, reader) {
                Ok(result) => { result }
                Err(err) => {
                    error!("Can't read Binary Data: {:?}", err);
                    return Err(DecodeError::BinaryData { cause: err });
                }
            };
            binary_data.push(byte);
        }
        trace!("Extracted Binary Data: {:?}", binary_data);
        return Ok(binary_data);
    }
}

pub struct FixedHeaderDecoder {}

impl FixedHeaderDecoder {
    pub fn new() -> Self {
        FixedHeaderDecoder {}
    }

    fn read_packet_type(&self, reader: &mut BitReader) -> DecodeResult<ControlPacketType> {
        trace!("FixedHeaderDecoder::read_packet_type");
        let packet_type_raw = match self.read_u8(4, reader) {
            Ok(result) => { result }
            Err(err) => { return Err(DecodeError::PacketType { cause: err }); }
        };
        let packet_type = ControlPacketType::from_u8(packet_type_raw);

        return match packet_type {
            None => {
                error!("Can't match Packet Type. byte value: {:?}", packet_type_raw);
                Err(DecodeError::PacketType { cause: ReadError::ExceededMaxValue { max: 15, current: packet_type_raw as u64 } })
            }
            Some(result) => {
                trace!("Extracted Packet Type: {:?}",  packet_type);
                Ok(result)
            }
        };
    }

    fn read_control_flags(&self, reader: &mut BitReader) -> DecodeResult<Vec<bool>> {
        trace!("FixedHeaderDecoder::read_control_flags");
        let flags = match self.read_booleans(4, reader) {
            Ok(result) => { result }
            Err(err) => { return Err(DecodeError::ControlFlags { cause: err }); }
        };
        trace!("Extracted Control Flags: {:?}", flags);
        return Ok(flags);
    }

    fn read_remaining_length(&self, reader: &mut BitReader) -> DecodeResult<u64> {
        trace!("FixedHeaderDecoder::read_remaining_length");
        let remaining_length = match self.read_variable_byte_integer(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Remaining Length: {:?}", err);
                return Err(DecodeError::RemainingLength { cause: err.cause() });
            }
        };
        trace!("Extracted Remaining Length: {:?}", remaining_length);
        return Ok(remaining_length);
    }

    async fn read_variable_byte_integer_as_buf(&self, reader: &mut MutexGuard<'_, OwnedReadHalf>) -> DecodeResult<Vec<u8>> {
        trace!("FixedHeaderDecoder::read_variable_byte_integer_from_stream");
        let mut multiplier: u64 = 1;
        let mut consumed_bytes = Vec::with_capacity(1);
        loop {
            let encoded_byte = match reader.read_u8().await {
                Ok(res) => {
                    consumed_bytes.push(res);
                    res
                }
                Err(err) => {
                    error!("Can't decode Variable Byte Integer: {:?}", err);
                    return Err(DecodeError::VariableByteInteger { cause: ReadError::InvalidData });
                }
            };
            // trace!("Encoded byte: {:?}", encoded_byte);
            if multiplier > 128 * 128 * 128 {
                error!("Can't decode Variable Byte Integer. Multiplier: {:?} ", multiplier);
                return Err(DecodeError::VariableByteInteger { cause: ReadError::ExceededMaxValue { current: multiplier, max: 128 * 128 * 128 } });
            }
            multiplier *= 128;
            // trace!("Multiplier: {:?}", multiplier);
            //trace!("Encoded byte & 128: {:?}", encoded_byte & 128);
            if (encoded_byte & 128) == 0 {
                break;
            }
        }
        return Ok(consumed_bytes);
    }

    pub async fn decode_from_stream(&self, stream: &mut MutexGuard<'_, OwnedReadHalf>) -> DecodeResult<FixedHeader> {
        debug!("FixedHeaderDecoder::decode_from_stream");
        let mut buffer = Vec::with_capacity(2);
        let first_byte = match stream.read_u8().await {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Fixed Header first byte from stream: {:?}", err);
                return match err.kind() {
                    ErrorKind::UnexpectedEof => {
                        Err(DecodeError::PacketType { cause: ReadError::ConnectionError })
                    }
                    ErrorKind::ConnectionAborted => {
                        Err(DecodeError::PacketType { cause: ReadError::ConnectionError })
                    }
                    ErrorKind::ConnectionRefused => {
                        Err(DecodeError::PacketType { cause: ReadError::ConnectionError })
                    }
                    ErrorKind::ConnectionReset => {
                        Err(DecodeError::PacketType { cause: ReadError::ConnectionError })
                    }
                    _ => {
                        Err(DecodeError::PacketType { cause: ReadError::IOError })
                    }
                };
            }
        };
        buffer.push(first_byte);

        let remaining_length_buffer = match self.read_variable_byte_integer_as_buf(stream).await {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Remaining Length bytes from stream: {:?}", err);
                return Err(DecodeError::RemainingLength { cause: err.cause() });
            }
        };
        buffer.put_slice(remaining_length_buffer.as_slice());
        let mut reader = BitReader::new(&buffer);
        return self.decode(&mut reader);
    }
}

impl Decoder<FixedHeader> for FixedHeaderDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<FixedHeader> {
        debug!("FixedHeaderDecoder::decode");
        let packet_type = self.read_packet_type(reader)?;
        return match packet_type {
            ControlPacketType::PUBLISH => {
                let dup_flag = match self.read_bool(reader) {
                    Ok(result) => { result }
                    Err(err) => {
                        return Err(DecodeError::DupFlag { cause: err });
                    }
                };
                trace!("Extracted DUP Flag: {:?}", dup_flag);
                let qos_level_raw = match self.read_u8(2, reader) {
                    Ok(result) => { result }
                    Err(err) => {
                        return Err(DecodeError::QoSLevel { cause: err });
                    }
                };
                let qos_level = match QoSLevel::from_u8(qos_level_raw) {
                    Some(result) => { result }
                    None => { return Err(DecodeError::QoSLevel { cause: ReadError::InvalidData }); }
                };
                trace!("Extracted QoS Level: {:?}", qos_level);

                let retain = match self.read_bool(reader) {
                    Ok(result) => { result }
                    Err(err) => {
                        return Err(DecodeError::RetainFlag { cause: err });
                    }
                };
                trace!("Extracted Retain Flag: {:?}", retain);

                let remaining_length = self.read_remaining_length(reader)?;
                Ok(FixedHeader::from_publish(dup_flag, qos_level, retain, remaining_length))
            }
            _ => {
                let control_flags = self.read_control_flags(reader)?;
                let remaining_length = self.read_remaining_length(reader)?;
                Ok(FixedHeader::new(packet_type, control_flags, remaining_length))
            }
        };
    }
}

pub struct VariableHeaderDecoder {
    fixed_header: FixedHeader,
}

impl VariableHeaderDecoder {
    pub fn new(fixed_header: FixedHeader) -> Self {
        VariableHeaderDecoder { fixed_header }
    }
}

impl VariableHeaderDecoder {
    fn read_protocol_name(&self, reader: &mut BitReader) -> DecodeResult<String> {
        trace!("VariableHeaderDecoder::read_protocol_name");
        let protocol_name = self.read_utf8_string(reader)?;
        trace!("Extracted Protocol Name: {:?}", protocol_name);
        return Ok(protocol_name);
    }

    fn read_protocol_version(&self, reader: &mut BitReader) -> DecodeResult<u8> {
        trace!("VariableHeaderDecoder::read_protocol_version");
        let protocol_version = match self.read_u8(8, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Protocol Version: {:?}", err);
                return Err(DecodeError::ProtocolVersion { cause: err });
            }
        };
        trace!("Extracted Protocol Version: {:?}", protocol_version);
        return Ok(protocol_version);
    }

    fn read_connect_flags(&self, reader: &mut BitReader) -> DecodeResult<ConnectFlags> {
        trace!("VariableHeaderDecoder::read_connect_flags");

        let username_flag = self.read_username_flag(reader)?;
        let password_flag = self.read_password_flag(reader)?;
        let will_retain_flag = self.read_will_retain_flag(reader)?;
        let will_qos = self.read_will_qos_flags(reader)?;
        let will_flag = self.read_will_flag(reader)?;
        let clean_start_flag = self.read_clean_start_flag(reader)?;
        let reserved_flag = self.read_reserved_flag(reader)?;
        return Ok(ConnectFlags::new(
            username_flag,
            password_flag,
            will_retain_flag,
            will_qos,
            will_flag,
            clean_start_flag,
            reserved_flag,
        ));
    }

    fn read_username_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_username_flag");

        let username_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Username Flag: {:?}", err);
                return Err(DecodeError::UsernameFlag { cause: err });
            }
        };
        trace!("Extracted User Name Flag: {:?}", username_flag);
        return Ok(username_flag);
    }

    fn read_password_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_password_flag");

        let password_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Password Flag: {:?}", err);
                return Err(DecodeError::PasswordFlag { cause: err });
            }
        };
        trace!("Extracted Password Flag: {:?}", password_flag);
        return Ok(password_flag);
    }

    fn read_will_retain_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_will_retain_flag");

        let will_retain_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will Retain Flag: {:?}", err);
                return Err(DecodeError::UsernameFlag { cause: err });
            }
        };
        trace!("Extracted Will Retain Flag: {:?}", will_retain_flag);
        return Ok(will_retain_flag);
    }

    fn read_will_qos_flags(&self, reader: &mut BitReader) -> DecodeResult<QoSLevel> {
        trace!("VariableHeaderDecoder::read_will_qos_flags");

        let will_qos_raw = match self.read_u8(2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will QoS Flag: {:?}", err);
                return Err(DecodeError::WillQoSFlag { cause: err });
            }
        };
        let will_qos = QoSLevel::from_u8(will_qos_raw);
        return match will_qos {
            None => {
                error!("Invalid Will QoS: {:?}", will_qos_raw);
                Err(DecodeError::WillQoSFlag { cause: ReadError::InvalidData })
            }
            Some(will_qos) => {
                trace!("Extracted Will QoS: {:?}", will_qos);
                Ok(will_qos)
            }
        };
    }

    fn read_clean_start_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_clean_start_flag");

        let clean_start = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Clean Start Flag: {:?}", err);
                return Err(DecodeError::CleanStartFlag { cause: err });
            }
        };
        trace!("Extracted Clean Start Flag: {:?}", clean_start);
        return Ok(clean_start);
    }


    fn read_will_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_will_flag");

        let will_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will Flag: {:?}", err);
                return Err(DecodeError::WillFlag { cause: err });
            }
        };
        trace!("Extracted Will Flag: {:?}", will_flag);
        return Ok(will_flag);
    }

    fn read_reserved_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_reserved_flag");

        let reserved_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Reserved Flag: {:?}", err);
                return Err(DecodeError::ReservedFlag { cause: err });
            }
        };
        trace!("Extracted Reserved Flag: {:?}", reserved_flag);
        return Ok(reserved_flag);
    }

    fn read_keep_alive(&self, reader: &mut BitReader) -> DecodeResult<u16> {
        trace!("VariableHeaderDecoder::read_keep_alive");

        let keep_alive = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Keep Alive: {:?}", err);
                return Err(DecodeError::KeepAlive { cause: err });
            }
        };
        //time interval measured in seconds
        trace!("Extracted Keep Alive: {:?}", keep_alive);
        return Ok(keep_alive);
    }
    fn read_packet_identifier(&self, reader: &mut BitReader) -> DecodeResult<u16> {
        let packet_identifier = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Packet Identifier: {:?}", err);
                return Err(DecodeError::PacketIdentifier { cause: err });
            }
        };
        Ok(packet_identifier)
    }

    fn read_reason_code(&self, reader: &mut BitReader) -> DecodeResult<ReasonCode> {
        return match self.read_u8(8, reader) {
            Ok(result) => {
                match ReasonCode::from_u8(result) {
                    Some(reason_code) => { Ok(reason_code) }
                    None => {
                        error!("Can't decode ReasonCode from value: {:?}", result);
                        Err(DecodeError::ReasonCode { cause: ReadError::InvalidData })
                    }
                }
            }
            Err(err) => {
                error!("Can't decode Packet Identifier: {:?}", err);
                return Err(DecodeError::ReasonCode { cause: err });
            }
        };
    }
}

impl Decoder<Option<VariableHeader>> for VariableHeaderDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Option<VariableHeader>> {
        debug!("VariableHeaderDecoder::decode");
        let property_decoder = PropertyDecoder::new();
        return Ok(match self.fixed_header.packet_type() {
            ControlPacketType::RESERVED => { None }
            ControlPacketType::CONNECT => {
                let start_position = reader.position();

                let protocol_name = self.read_protocol_name(reader)?;
                let protocol_version = self.read_protocol_version(reader)?;
                let connect_flags = self.read_connect_flags(reader)?;
                let keep_alive = self.read_keep_alive(reader)?;
                let properties = property_decoder.decode(reader)?;
                trace!("Variable Header consumed {:?} bytes from stream", (reader.position() - start_position) / 8);


                Some(VariableHeader::from_connect(Some(protocol_name), Some(protocol_version), Some(connect_flags), Some(keep_alive), properties))
            }
            ControlPacketType::CONNACK => { None }
            ControlPacketType::PUBLISH => {
                let topic_name = match self.read_utf8_string(reader) {
                    Ok(result) => { result }
                    Err(err) => {
                        error!("Can't decode Topic Name: {:?}", err);
                        return Err(DecodeError::TopicName { cause: err.cause() });
                    }
                };
                trace!("Extracted Topic Name: {:?}", topic_name);
                let mut packet_identifier = None;

                match self.fixed_header.qos_level() {
                    QoSLevel::AtMostOnce => {}
                    _ => {
                        packet_identifier = Some(self.read_packet_identifier(reader)?);
                        trace!("Extracted Packet Identifier: {:?}", packet_identifier);
                    }
                }

                let properties = property_decoder.decode(reader)?;
                Some(VariableHeader::from_publish(packet_identifier, Some(topic_name), properties))
            }
            ControlPacketType::PUBACK => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBREC => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBREL => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBCOMP => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::SUBSCRIBE => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let properties = property_decoder.decode(reader)?;
                Some(VariableHeader::from_sub_unsub(Some(packet_identifier), properties))
            }
            ControlPacketType::SUBACK => { None }
            ControlPacketType::UNSUBSCRIBE => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let properties = property_decoder.decode(reader)?;
                Some(VariableHeader::from_sub_unsub(Some(packet_identifier), properties))
            }
            ControlPacketType::UNSUBACK => { None }
            ControlPacketType::PINGREQ => { None }
            ControlPacketType::PINGRESP => { None }
            ControlPacketType::DISCONNECT => {
                let reason_code = self.read_reason_code(reader)?;
                let properties = property_decoder.decode(reader)?;
                Some(VariableHeader::from_disconnect(reason_code, properties))
            }
            ControlPacketType::AUTH => { None }
        });
    }
}

struct PropertyDecoder {}


impl PropertyDecoder {
    pub fn new() -> Self {
        PropertyDecoder {}
    }

    fn read_property_length(&self, reader: &mut BitReader) -> DecodeResult<u64> {
        trace!("PropertyDecoder::read_property_length");

        let property_length = match self.read_variable_byte_integer(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Property Length: {:?}", err);
                return Err(DecodeError::PropertyLength { cause: err.cause() });
            }
        };
        trace!("Extracted Property Length: {:?}", property_length);
        return Ok(property_length);
    }

    fn read_property(&self, identifier: u64, reader: &mut BitReader) -> DecodeResult<Option<Property>> {
        let map_error = |err: ReadError| {
            error!("Can't decode property: {:?}", err);
            return Err(DecodeError::Property { cause: err });
        };
        return match identifier {
            0 => {
                let value = self.read_variable_byte_integer(reader)?;
                Ok(Some(Property::SubscriptionIdentifier(value)))
            }
            1 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::PayloadFormatIndicator(value)))
            }
            2 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MessageExpiryInterval(value)))
            }
            3 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ContentType(value)))
            }
            8 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ResponseTopic(value)))
            }
            9 => {
                let value = self.read_binary_data(reader)?;
                Ok(Some(Property::CorrelationData(value)))
            }
            11 => {
                let value = self.read_variable_byte_integer(reader)?;
                Ok(Some(Property::SubscriptionIdentifier(value)))
            }
            17 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SessionExpiryInterval(value)))
            }
            18 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::AssignedClientIdentifier(value)))
            }
            19 => {
                let value = match self.read_u8(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::ServerKeepAlive(value)))
            }
            21 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::AuthenticationMethod(value)))
            }
            22 => {
                let value = self.read_binary_data(reader)?;
                Ok(Some(Property::AuthenticationData(value)))
            }
            23 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RequestProblemInformation(value)))
            }
            24 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::WillDelayInterval(value)))
            }
            25 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RequestResponseInformation(value)))
            }
            26 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ResponseInformation(value)))
            }
            28 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ServerReference(value)))
            }
            31 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ReasonString(value)))
            }
            33 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::ReceiveMaximum(value)))
            }
            34 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::TopicAliasMaximum(value)))
            }
            35 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::TopicAlias(value)))
            }
            36 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MaximumQoS(value)))
            }
            37 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RetainAvailable(value)))
            }
            38 => {
                let key = self.read_utf8_string(reader)?;
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::UserProperty(key, value)))
            }
            39 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MaximumPacketSize(value)))
            }
            40 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::WildcardSubscriptionAvailable(value)))
            }
            41 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SubscriptionIdentifierAvailable(value)))
            }
            42 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SharedSubscriptionAvailable(value)))
            }
            unknown_value => {
                error!("Can't build Property. Unknown identifier {:?}", unknown_value);
                Err(DecodeError::UnknownProperty { cause: ReadError::InvalidData })
            }
        };
    }
}

impl Decoder<Vec<Property>> for PropertyDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Vec<Property>> {
        debug!("PropertyDecoder::decode");
        let mut properties: Vec<Property> = Vec::new();
        let start_position = reader.position();
        let mut properties_byte_size = self.read_property_length(reader)?;
        //Property Length in bytes
        trace!("Properties Byte Size: {:?}", properties_byte_size);

        while properties_byte_size > 0 {
            let properties_start = reader.position();
            let identifier = self.read_variable_byte_integer(reader)?;
            trace!("Property Identifier: {:?}", identifier);
            let property = self.read_property(identifier, reader)?;
            trace!("Extracted Property: {:?}", property);
            if property.is_some() {
                properties.push(property.unwrap());
            }
            //I need to check how many bytes Property Identifier and their values consumed from stream
            let consumed_bytes = (reader.position() - properties_start) / 8;
            properties_byte_size = properties_byte_size - consumed_bytes;
            trace!("Consumed bytes: {:?}. Remaining properties bytes: {:?}", consumed_bytes, properties_byte_size);
        }
        trace!("Properties consumed {:?} bytes from stream", (reader.position() - start_position) / 8);

        return Ok(properties);
    }
}

pub struct PayloadDecoder {
    packet_type: ControlPacketType,
    variable_header: Option<VariableHeader>,
}

impl PayloadDecoder {
    pub fn variable_header(&self) -> &VariableHeader {
        self.variable_header.as_ref().unwrap()
    }
}


impl PayloadDecoder {
    pub fn new(packet_type: ControlPacketType, variable_header: Option<VariableHeader>) -> Self {
        PayloadDecoder { packet_type, variable_header }
    }

    fn read_topic_path(&self, reader: &mut BitReader) -> DecodeResult<String> {
        trace!("PayloadDecoder::read_topic_path");
        let topic_path = match self.read_utf8_string(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Topic Filter: {:?}", err);
                return Err(DecodeError::TopicFilter { cause: err.cause() });
            }
        };
        Ok(topic_path)
    }

    fn read_topic_filter(&self, reader: &mut BitReader) -> DecodeResult<TopicFilter> {
        trace!("PayloadDecoder::read_topic_filter");
        let topic_filter = self.read_topic_path(reader)?;
        trace!("Extracted Topic Filter: {:?}", topic_filter);
        let reserved_bits = match self.read_booleans(2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Reserved Bits: {:?}", err);
                return Err(DecodeError::ReservedFlag { cause: err });
            }
        };
        trace!("Extracted Reserved Bits: {:?}", reserved_bits);

        let retain_handling = match self.read_u8(2, reader) {
            Ok(retain_handling) => {
                match RetainHandling::from_u8(retain_handling) {
                    Some(retain_handling) => { retain_handling }
                    None => {
                        error!("Can't decode RetainHandling from value: {:?}", retain_handling);
                        return Err(DecodeError::RetainHandling { cause: ReadError::ExceededMaxValue { current: retain_handling as u64, max: 2 } });
                    }
                }
            }
            Err(err) => {
                error!("Can't read RetainHandling from: {:?}", err);
                return Err(DecodeError::RetainHandling { cause: err });
            }
        };
        trace!("Extracted Retain Handling: {:?}", retain_handling);

        let retain_as_published = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Retain As Published: {:?}", err);
                return Err(DecodeError::RetainAsPublished { cause: err });
            }
        };
        trace!("Extracted Retain As Published: {:?}", retain_as_published);

        let no_local = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read No Local: {:?}", err);
                return Err(DecodeError::NoLocal { cause: err });
            }
        };
        trace!("Extracted No Local: {:?}", no_local);

        let maximum_qos = match self.read_u8(2, reader) {
            Ok(qos_level) => {
                match QoSLevel::from_u8(qos_level) {
                    Some(qos_level) => { qos_level }
                    None => {
                        error!("Can't decode MaximumQoS from value: {:?}", qos_level);
                        return Err(DecodeError::MaximumQoS { cause: ReadError::ExceededMaxValue { current: qos_level as u64, max: 2 } });
                    }
                }
            }
            Err(err) => {
                error!("Can't read MaximumQoS: {:?}", err);
                return Err(DecodeError::MaximumQoS { cause: err });
            }
        };
        trace!("Extracted Maximum QoS: {:?}", maximum_qos);

        Ok(TopicFilter::from_subscribe(topic_filter, maximum_qos, no_local, retain_as_published, retain_handling, reserved_bits))
    }
}

impl Decoder<Option<Payload>> for PayloadDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Option<Payload>> {
        debug!("PayloadDecoder::decode");
        return Ok(match self.packet_type {
            ControlPacketType::RESERVED => { None }
            ControlPacketType::CONNECT => {
                let client_id = self.read_utf8_string(reader)?;
                trace!("Extracted Client ID: {:?}", client_id);

                let connect_flags = self.variable_header().connect_flags();

                let mut will_properties: Option<Vec<Property>> = None;
                let mut will_topic: Option<String> = None;
                let mut will_payload: Option<Vec<u8>> = None;

                if connect_flags.will_flag() {
                    let property_decoder = PropertyDecoder::new();
                    will_properties = Option::from(property_decoder.decode(reader)?);
                    trace!("Extracted Will Properties: {:?}", will_properties);

                    will_topic = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Will Topic: {:?}", will_topic);

                    will_payload = Option::from(self.read_binary_data(reader)?);
                    trace!("Extracted Will Payload: {:?}", will_payload);
                }

                let mut username: Option<String> = None;
                if connect_flags.username_flag() {
                    username = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Username: {:?}", username);
                }

                let mut password: Option<String> = None;
                if connect_flags.password_flag() {
                    password = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Password: {:?}", password);
                }

                Option::from(Payload::from_connect(Some(client_id), will_properties, will_topic, will_payload, username, password))
            }
            ControlPacketType::CONNACK => { None }
            ControlPacketType::PUBLISH => {
                let mut data = Vec::with_capacity((reader.remaining() / 8) as usize);
                while data.len() != data.capacity() {
                    data.push(
                        match reader.read_u8(8) {
                            Ok(result) => { result }
                            Err(err) => {
                                error!("Can't read payload: {:?}", err);
                                return match err {
                                    BitReaderError::NotEnoughData {
                                        position,
                                        length,
                                        requested, } => {
                                        Err(DecodeError::Payload { cause: ReadError::NotEnoughData { position, length, requested } })
                                    }
                                    BitReaderError::TooManyBitsForType {
                                        position,
                                        requested,
                                        allowed, } => {
                                        Err(DecodeError::Payload { cause: ReadError::TooManyBitsForType { position, requested, allowed } })
                                    }
                                };
                            }
                        })
                }
                Option::from(Payload::from_publish(Option::from(data)))
            }
            ControlPacketType::PUBACK => { None }
            ControlPacketType::PUBREC => { None }
            ControlPacketType::PUBREL => { None }
            ControlPacketType::PUBCOMP => { None }
            ControlPacketType::SUBSCRIBE => {
                let mut topic_filters = Vec::new();
                while reader.remaining() != 0 {
                    let topic_filter = self.read_topic_filter(reader)?;
                    topic_filters.push(topic_filter);
                }
                Option::from(Payload::from_sub_unsub(topic_filters))
            }
            ControlPacketType::SUBACK => { None }
            ControlPacketType::UNSUBSCRIBE => {
                let mut topic_filters = Vec::new();
                while reader.remaining() != 0 {
                    let topic_path = self.read_topic_path(reader)?;
                    topic_filters.push(TopicFilter::from_unsubscribe(topic_path));
                }
                Option::from(Payload::from_sub_unsub(topic_filters))
            }
            ControlPacketType::UNSUBACK => { None }
            ControlPacketType::PINGREQ => { None }
            ControlPacketType::PINGRESP => { None }
            ControlPacketType::DISCONNECT => { None }
            ControlPacketType::AUTH => { None }
        });
    }
}

