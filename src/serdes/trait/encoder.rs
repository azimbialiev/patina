
use bytes::{BufMut, BytesMut};
use log::trace;
use crate::serdes::serializer::error::{EncodeError, EncodeResult};


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




