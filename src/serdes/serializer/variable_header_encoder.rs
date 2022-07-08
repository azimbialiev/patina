use std::borrow::BorrowMut;

use bytes::{BufMut, BytesMut};
use log::{debug, trace};

use crate::model::fixed_header::ControlPacketType;
use crate::model::reason_code::ReasonCode;
use crate::model::variable_header::{ConnectAcknowledgeFlags, VariableHeader};
use crate::serdes::r#trait::encoder::{Encoder, LengthCalculator, OptEncoder};
use crate::serdes::serializer::error::EncodeResult;
use crate::serdes::serializer::property_encoder::PropertyEncoder;

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