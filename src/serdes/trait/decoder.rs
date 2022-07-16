use bitreader::{BitReader, BitReaderError};
use log::{error, trace};

use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError, ReadResult};

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




