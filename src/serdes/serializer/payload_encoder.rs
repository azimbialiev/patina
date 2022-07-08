use std::borrow::BorrowMut;

use bytes::{BufMut, BytesMut};
use log::{debug, trace};

use crate::model::fixed_header::ControlPacketType;
use crate::model::payload::Payload;
use crate::model::reason_code::ReasonCode;
use crate::serdes::r#trait::encoder::{Encoder, LengthCalculator, OptEncoder};
use crate::serdes::serializer::error::EncodeResult;

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
