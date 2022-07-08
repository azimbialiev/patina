use std::io::ErrorKind;
use std::ops::Deref;
use std::sync::Arc;

use bitreader::BitReader;
use bytes::BytesMut;
use log::{debug, error, trace};
use metered::{*};
use nameof::name_of_type;
use serde::Serializer;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::MutexGuard;

use crate::model::control_packet::ControlPacket;
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::deserializer::fixed_header_decoder::FixedHeaderDecoder;
use crate::serdes::deserializer::payload_decoder::PayloadDecoder;
use crate::serdes::deserializer::variable_header_decoder::VariableHeaderDecoder;
use crate::serdes::r#trait::decoder::Decoder;

#[derive(Default, Clone, Debug)]
pub struct MqttDecoder(Arc<MqttDecoderImpl>);

impl Deref for MqttDecoder {
    type Target = MqttDecoderImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl serde::Serialize for MqttDecoder{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        return self.deref().serialize(serializer);
    }
}

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct MqttDecoderImpl {
    pub(crate) metrics: MqttDecoderMetrics,

}

#[metered(registry = MqttDecoderMetrics)]
impl MqttDecoderImpl {

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub(crate) async fn decode_packet(&self, mut stream: MutexGuard<'_, OwnedReadHalf>) -> DecodeResult<ControlPacket> {
        debug!("{}::decode_packet", name_of_type!(MqttDecoder));
        let fixed_header_decoder = FixedHeaderDecoder::new();
        let fixed_header = fixed_header_decoder.decode_from_stream(&mut stream).await?;

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

            let mut reader = BitReader::new(buffer.as_ref());

            let variable_header_decoder = VariableHeaderDecoder::new(fixed_header.clone());
            variable_header = variable_header_decoder.decode(&mut reader)?;
            let payload_decoder = PayloadDecoder::new(fixed_header.packet_type(), variable_header.clone());
            payload = payload_decoder.decode(&mut reader)?;
        }

        let control_packet = ControlPacket::new(fixed_header, variable_header, payload);
        debug!("ControlPacket: {:?}", control_packet);
        return Ok(control_packet);
    }
}