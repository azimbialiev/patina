use std::borrow::BorrowMut;

use bytes::{BufMut, BytesMut};
use log::{debug, error, trace};

use crate::model::fixed_header::{ControlPacketType, FixedHeader};
use crate::model::qos_level::QoSLevel;
use crate::serdes::r#trait::encoder::{Encoder, LengthCalculator};
use crate::serdes::serializer::error::{EncodeError, EncodeResult};

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