use std::borrow::BorrowMut;

use bytes::{BufMut, BytesMut};
use log::{debug, trace};

use crate::model::variable_header::Property;
use crate::serdes::r#trait::encoder::{Encoder, LengthCalculator};
use crate::serdes::serializer::error::EncodeResult;

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
