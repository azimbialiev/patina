use std::io::ErrorKind;

use bitreader::BitReader;
use bytes::BytesMut;
use log::{debug, error, trace};
use metered::{*};
use serde::Serializer;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

use crate::model::control_packet::ControlPacket;
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::deserializer::fixed_header_decoder::FixedHeaderDecoder;
use crate::serdes::deserializer::payload_decoder::PayloadDecoder;
use crate::serdes::deserializer::variable_header_decoder::VariableHeaderDecoder;

#[derive(Default, Debug)]
pub struct MqttDecoder {
    pub(crate) metrics: MqttDecoderMetrics,
    pub(crate) fixed_header_decoder: FixedHeaderDecoder,
    pub(crate) variable_header_decoder: VariableHeaderDecoder,
    pub(crate) payload_decoder: PayloadDecoder,
}

#[metered(registry = MqttDecoderMetrics)]
impl MqttDecoder {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub(crate) async fn decode_packet(&self, mut stream: OwnedReadHalf) -> DecodeResult<(OwnedReadHalf, ControlPacket)> {
        debug!("START decode_packet");
        let fixed_header = self.fixed_header_decoder.decode_from_stream(&mut stream).await?;

        let mut buffer = BytesMut::with_capacity(fixed_header.remaining_length() as usize);
        debug!("Remaining packet length: {:?}", fixed_header.remaining_length());
        let mut variable_header = None;
        let mut payload = None;
        if fixed_header.remaining_length() > 0 {
            match stream.read_buf(&mut buffer).await {
                Ok(bytes_read) => {
                    trace!("Read {:?} bytes from stream", bytes_read);
                }
                Err(err) => {
                    error!("Can't read VariableHeader and Payload bytes from stream: {:?}", err);
                    return match err.kind() {
                        ErrorKind::UnexpectedEof => {
                            Err(DecodeError::VariableHeaderAndPayload { cause: ReadError::ConnectionError })
                        }
                        ErrorKind::ConnectionAborted => {
                            Err(DecodeError::VariableHeaderAndPayload { cause: ReadError::ConnectionError })
                        }
                        ErrorKind::ConnectionRefused => {
                            Err(DecodeError::VariableHeaderAndPayload { cause: ReadError::ConnectionError })
                        }
                        ErrorKind::ConnectionReset => {
                            Err(DecodeError::VariableHeaderAndPayload { cause: ReadError::ConnectionError })
                        }
                        _ => {
                            Err(DecodeError::VariableHeaderAndPayload { cause: ReadError::IOError })
                        }
                    };
                }
            };

            let mut reader = BitReader::new(&buffer);

            variable_header = self.variable_header_decoder.decode_with_header(&fixed_header, &mut reader)?;
            if let Some(_variable_header) = variable_header{
                payload = self.payload_decoder.decode_with_headers(&fixed_header, &_variable_header,&mut reader)?;
                variable_header = Some(_variable_header);

            }
        }

        let control_packet = ControlPacket::new(fixed_header, variable_header, payload);
        debug!("ControlPacket: {:?}", control_packet);
        return Ok((stream, control_packet));
    }
}