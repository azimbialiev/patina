use core::{fmt, result};
use std::borrow::{ BorrowMut};

use bytes::{BufMut, BytesMut};
use log::{debug, error, trace};

use crate::serdes::mqtt::{ConnectAcknowledgeFlags, ControlPacketType, FixedHeader, Payload, Property, QoSLevel, ReasonCode, VariableHeader};

pub type EncodeResult<T> = result::Result<T, EncodeError>;

#[derive(Debug, PartialEq, Clone)]
pub enum EncodeError {
    NotEnoughData,
    ExceededMaxLength,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //self.description().fmt(fmt)
        match *self {
            EncodeError::NotEnoughData => write!(fmt, "EncodeError::NotEnoughData"),
            EncodeError::ExceededMaxLength => write!(fmt, "EncodeError::ExceededMaxLength"),
        }
    }
}

pub trait LengthCalculator<T>: Encoder<T> {
    fn calculate_length(&mut self, item: &T) -> usize {
        trace!("LengthCalculator::calculate_length");
        self.internal_buffer_mut().clear();
        let buffer = &mut BytesMut::new();
        self.encode(item, buffer).expect("panic self.encode");
        self.internal_buffer_mut().put_slice(buffer);
        let length = self.internal_buffer_mut().len();
        trace!("Item Length: {:?}", length);
        return length;
    }
}

pub trait OptEncoder<T>: Encoder<T> {
    fn encode_opt(&mut self, item: Option<&T>, buffer: &mut BytesMut) -> EncodeResult<()> {
        if item.is_some() {
            return self.encode(item.unwrap(), buffer);
        }
        Ok(())
    }
}


pub trait Encoder<T> {
    fn encode(&mut self, item: &T, buffer: &mut BytesMut) -> EncodeResult<()>;

    fn internal_buffer_mut(&mut self) -> &mut BytesMut;

    fn write_variable_byte_integer(&mut self, mut value: u64, buffer: &mut BytesMut) -> EncodeResult<()> {
        trace!("Encoder::write_variable_byte_integer");
        let start = buffer.len();
        loop {
            let mut encoded_byte = value % 128;
            value = value / 128;

            if value > 0 {
                encoded_byte = encoded_byte | 128;
            }
            trace!("Writing EncodedVariableInteger: {:#04X?}", encoded_byte);
            buffer.put_u8(encoded_byte as u8);
            if value <= 0 {
                break;
            }
        }
        trace!("Encoded VariableByteInteger length: {:?}", buffer.len() - start);
        return Ok(());
    }

    fn write_utf8_encoded_string(&mut self, value: &String, buffer: &mut BytesMut) -> EncodeResult<()> {
        trace!("Encoder::write_utf8_encoded_string");
        if value.len() > u16::MAX as usize {
            return Err(EncodeError::ExceededMaxLength);
        }
        buffer.put_u16(value.len() as u16);
        buffer.put_slice(value.as_bytes());
        Ok(())
    }

    fn write_binary_data(&mut self, value: Vec<u8>, buffer: &mut BytesMut) -> EncodeResult<()> {
        trace!("Encoder::write_binary_data");
        if value.len() > u16::MAX as usize {
            return Err(EncodeError::ExceededMaxLength);
        }
        buffer.put_u16(value.len() as u16);
        buffer.put_slice(&value);
        Ok(())
    }
}

pub struct FixedHeaderEncoder {
    internal_buffer: BytesMut,

}

impl FixedHeaderEncoder {
    pub(crate) fn new() -> Self {
        trace!("FixedHeaderEncoder::new");
        let internal_buffer = BytesMut::new();
        FixedHeaderEncoder { internal_buffer }
    }
}

impl FixedHeaderEncoder {
    fn encode_packet_type(&mut self, packet_type: ControlPacketType) -> u8 {
        trace!("FixedHeaderEncoder::encode_packet_type");
        let value: u8 = packet_type.as_u8();
        trace!("Encoded PacketType: {:#04X?}", value);
        return value;
    }

    fn encode_control_flags(&mut self, mut first_byte: u8, control_flags: &Vec<bool>) -> EncodeResult<u8> {
        trace!("FixedHeaderEncoder::encode_control_flags");
        if control_flags.len() != 4 {
            error!("Encode control flags requires exactly 4 bits. Found: {:?}", control_flags.len());
            return Err(EncodeError::NotEnoughData);
        }

        for i in 0..(control_flags.len() - 1) {
            let bit_pos = i; //Control flags need to be in the 4 MSBs
            first_byte = (if control_flags[i] { 1 } else { 0 } << bit_pos) | first_byte;
        }
        trace!("Encoded ControlFlags (with PacketType): {:#04X?}", first_byte);
        return Ok(first_byte);
    }

    fn encode_publish_flags(&mut self, mut first_byte: u8, dup_flag: bool, qos_level: QoSLevel, retain: bool) -> EncodeResult<u8> {
        trace!("FixedHeaderEncoder::encode_publish_flags");
        first_byte = (if dup_flag { 1 } else { 0 } << 0) | first_byte;
        let qos_flags = qos_level.to_bool();
        first_byte = (if qos_flags.1 { 1 } else { 0 } << 1) | first_byte;
        first_byte = (if qos_flags.0 { 1 } else { 0 } << 2) | first_byte;
        first_byte = (if retain { 1 } else { 0 } << 3) | first_byte;
        return Ok(first_byte);
    }
}

impl LengthCalculator<(&FixedHeader, u64)> for FixedHeaderEncoder {}

impl Encoder<(&FixedHeader, u64)> for FixedHeaderEncoder {
    fn encode(&mut self, item: &(&FixedHeader, u64), buffer: &mut BytesMut) -> EncodeResult<()> {
        debug!("FixedHeaderEncoder::encode");
        let fixed_header = item.0;
        let remaining_length = item.1;
        if !self.internal_buffer.is_empty() {
            trace!("FixedHeaderEncoder Internal buffer is not empty. Length: {:?}", self.internal_buffer.len() );
            buffer.put_slice(&self.internal_buffer);
            return Ok(());
        }
        let mut first_byte = self.encode_packet_type(fixed_header.packet_type());
        match fixed_header.packet_type() {
            ControlPacketType::PUBLISH => {
                first_byte = self.encode_publish_flags(first_byte, *fixed_header.dup_flag(), *fixed_header.qos_level(), *fixed_header.retain())?;
            }
            _ => {
                first_byte = self.encode_control_flags(first_byte, fixed_header.control_flags())?;
            }
        }
        buffer.put_u8(first_byte);
        self.write_variable_byte_integer(remaining_length, buffer)?;
        Ok(())
    }

    fn internal_buffer_mut(&mut self) -> &mut BytesMut {
        self.internal_buffer.borrow_mut()
    }
}

pub struct PropertyEncoder {
    internal_buffer: BytesMut,

}

impl PropertyEncoder {
    pub fn new() -> Self {
        debug!("PropertyEncoder::new");

        let internal_buffer = BytesMut::new();
        PropertyEncoder { internal_buffer }
    }
}

impl LengthCalculator<Vec<Property>> for PropertyEncoder {}

impl Encoder<Vec<Property>> for PropertyEncoder {
    fn encode(&mut self, item: &Vec<Property>, buffer: &mut BytesMut) -> EncodeResult<()> {
        trace!("PropertyEncoder::encode");
        if !self.internal_buffer.is_empty() {
            trace!("PropertyEncoder Internal buffer is not empty. Length: {:?}", self.internal_buffer.len() );
            buffer.put_slice(&self.internal_buffer);
            return Ok(());
        }
        let length = 0;
        //TODO Implement properties encoding
        self.write_variable_byte_integer(length, buffer)?;
        Ok(())
    }

    fn internal_buffer_mut(&mut self) -> &mut BytesMut {
        self.internal_buffer.borrow_mut()
    }
}

pub struct VariableHeaderEncoder {
    packet_type: ControlPacketType,
    internal_buffer: BytesMut,
}

impl VariableHeaderEncoder {
    pub(crate) fn new(packet_type: ControlPacketType) -> Self {
        debug!("VariableHeaderEncoder::new");
        let internal_buffer = BytesMut::new();
        VariableHeaderEncoder { packet_type, internal_buffer }
    }

    fn encode_connect_acknowledge_flag(&self, flag: &ConnectAcknowledgeFlags, buffer: &mut BytesMut) {
        trace!("VariableHeaderEncoder::encode_connect_acknowledge_flag");
        let byte: u8 = (if flag.session_present() { 1 } else { 0 } << 0) | 0x00;
        trace!("Encoded Connection Acknowledge Flag: {:#04X?}", byte);
        buffer.put_u8(byte);
    }

    fn encode_reason_code(&self, reason_code: Option<&ReasonCode>, buffer: &mut BytesMut) {
        trace!("VariableHeaderEncoder::encode_connect_reason_code");
        if reason_code.is_some() {
            let value: u8 = reason_code.unwrap().as_u8();
            buffer.put_u8(value);
            trace!("Encoded Connect Reason Code: {:#04X?}", value);
        }
    }

    fn encode_packet_identifier(&self, packet_identifier: u16, buffer: &mut BytesMut) {
        trace!("VariableHeaderEncoder::encode_packet_identifier");
        buffer.put_u16(packet_identifier);
    }
}

impl LengthCalculator<VariableHeader> for VariableHeaderEncoder {}

impl OptEncoder<VariableHeader> for VariableHeaderEncoder {}

impl Encoder<VariableHeader> for VariableHeaderEncoder {
    fn encode(&mut self, item: &VariableHeader, buffer: &mut BytesMut) -> EncodeResult<()> {
        debug!("VariableHeaderEncoder::encode");
        if !self.internal_buffer.is_empty() {
            trace!("VariableHeaderEncoder Internal buffer is not empty. Length: {:?}", self.internal_buffer.len() );
            buffer.put_slice(&self.internal_buffer);
            return Ok(());
        }

        let mut property_encoder = PropertyEncoder::new();

        match self.packet_type {
            ControlPacketType::RESERVED => {}
            ControlPacketType::CONNECT => {}
            ControlPacketType::CONNACK => {
                self.encode_connect_acknowledge_flag(item.connect_acknowledge_flags(), buffer);
                self.encode_reason_code(item.reason_code(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PUBLISH => {
                self.write_utf8_encoded_string(item.topic_name(), buffer).expect("can't encode utf8 string");
                if item.packet_identifier_opt().is_some() {
                    self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                }
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PUBACK => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                self.encode_reason_code(item.reason_code(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PUBREC => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                self.encode_reason_code(item.reason_code(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PUBREL => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                self.encode_reason_code(item.reason_code(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PUBCOMP => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                self.encode_reason_code(item.reason_code(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::SUBSCRIBE => {}
            ControlPacketType::SUBACK => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::UNSUBSCRIBE => {}
            ControlPacketType::UNSUBACK => {
                self.encode_packet_identifier(item.packet_identifier_opt().unwrap(), buffer);
                property_encoder.encode(&item.properties(), buffer).expect("encode");
            }
            ControlPacketType::PINGREQ => {}
            ControlPacketType::PINGRESP => {}
            ControlPacketType::DISCONNECT => {}
            ControlPacketType::AUTH => {}
        }
        Ok(())
    }

    fn internal_buffer_mut(&mut self) -> &mut BytesMut {
        self.internal_buffer.borrow_mut()
    }
}

pub struct PayloadEncoder {
    packet_type: ControlPacketType,
    internal_buffer: BytesMut,

}

impl LengthCalculator<Payload> for PayloadEncoder {}

impl OptEncoder<Payload> for PayloadEncoder {}

impl PayloadEncoder {
    pub(crate) fn new(packet_type: ControlPacketType) -> Self {
        debug!("PayloadEncoder::new");
        let internal_buffer = BytesMut::new();
        PayloadEncoder { packet_type, internal_buffer }
    }

    pub fn calculate_length(&mut self, item: &Payload) -> usize {
        trace!("VariableHeaderEncoder::calculate_length");
        self.internal_buffer.clear();
        let buffer = &mut BytesMut::new();
        self.encode(item, buffer).expect("encode");
        self.internal_buffer.put_slice(buffer);
        let length = self.internal_buffer.len();
        trace!("VariableHeaderLength: {:?}", length);
        return length;
    }

    fn encode_reason_code(&self, reason_code: &ReasonCode, buffer: &mut BytesMut) {
        trace!("PayloadEncoder::encode_reason_code");

        let value: u8 = reason_code.as_u8();
        trace!("Encoded Reason Code: {:#04X?}", value);

        buffer.put_u8(value);
    }
}

impl Encoder<Payload> for PayloadEncoder {
    fn encode(&mut self, item: &Payload, buffer: &mut BytesMut) -> EncodeResult<()> {
        debug!("PayloadEncoder::encode");
        if !self.internal_buffer.is_empty() {
            trace!("PayloadEncoder Internal buffer is not empty. Length: {:?}", self.internal_buffer.len() );
            buffer.put_slice(&self.internal_buffer);
            return Ok(());
        }
        match self.packet_type {
            ControlPacketType::RESERVED => {}
            ControlPacketType::CONNECT => {}
            ControlPacketType::CONNACK => {}
            ControlPacketType::PUBLISH => {
                buffer.put_slice(item.data());
            }
            ControlPacketType::PUBACK => {}
            ControlPacketType::PUBREC => {}
            ControlPacketType::PUBREL => {}
            ControlPacketType::PUBCOMP => {}
            ControlPacketType::SUBSCRIBE => {}
            ControlPacketType::SUBACK => {
                for reason_code in item.reason_codes() {
                    self.encode_reason_code(reason_code, buffer);
                }
            }
            ControlPacketType::UNSUBSCRIBE => {}
            ControlPacketType::UNSUBACK => {
                for reason_code in item.reason_codes() {
                    self.encode_reason_code(reason_code, buffer);
                }
            }
            ControlPacketType::PINGREQ => {}
            ControlPacketType::PINGRESP => {}
            ControlPacketType::DISCONNECT => {}
            ControlPacketType::AUTH => {}
        }
        Ok(())
    }

    fn internal_buffer_mut(&mut self) -> &mut BytesMut {
        self.internal_buffer.borrow_mut()
    }
}


