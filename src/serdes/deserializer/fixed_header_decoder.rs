use std::io::ErrorKind;

use bitreader::BitReader;
use bytes::BufMut;
use log::{debug, error, trace};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::MutexGuard;

use crate::model::fixed_header::{ControlPacketType, FixedHeader};
use crate::model::qos_level::QoSLevel;
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::r#trait::decoder::Decoder;

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