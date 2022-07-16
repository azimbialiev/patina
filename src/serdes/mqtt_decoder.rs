use std::io::ErrorKind;
use std::ops::Deref;
use std::sync::Arc;

use bitreader::BitReader;
use bytes::BytesMut;
use log::{debug, error, trace};
use metered::{*};
use nameof::name_of_type;
use serde::Serializer;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::Mutex;

use crate::model::control_packet::ControlPacket;
use crate::model::payload::Payload;
use crate::model::variable_header::VariableHeader;
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::deserializer::fixed_header_decoder::FixedHeaderDecoder;
use crate::serdes::deserializer::payload_decoder::PayloadDecoder;
use crate::serdes::deserializer::variable_header_decoder::VariableHeaderDecoder;
use crate::serdes::r#trait::decoder::Decoder;



#[derive(Default, Debug)]
pub struct MqttDecoder {
    pub(crate) metrics: MqttDecoderMetrics,

}

#[metered(registry = MqttDecoderMetrics)]
impl MqttDecoder {

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub(crate) async fn decode_packet(&self, mut stream: OwnedReadHalf) -> DecodeResult<(OwnedReadHalf, ControlPacket)> {
        debug!("START decode_packet");
        let fixed_header_decoder = FixedHeaderDecoder::new();
        let fixed_header = Box::new(fixed_header_decoder.decode_from_stream(&mut stream).await?);

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

            let _fixed_header = fixed_header.clone();
            let res: (Option<VariableHeader>, Option<Payload>) = tokio::task::spawn_blocking(move || {
                let mut reader = BitReader::new(&buffer);

                let variable_header_decoder = VariableHeaderDecoder::new(_fixed_header.clone());
                let variable_header = variable_header_decoder.decode(&mut reader)?;
                let payload_decoder = PayloadDecoder::new(_fixed_header.packet_type(), variable_header.clone());
                let payload = payload_decoder.decode(&mut reader)?;
                Ok((variable_header, payload))
            }).await.expect("panic spawn blocking")?;

            variable_header = res.0;
            payload = res.1;
        }

        let control_packet = ControlPacket::new(fixed_header.deref().to_owned(), variable_header, payload);
        debug!("ControlPacket: {:?}", control_packet);
        return Ok((stream, control_packet));
    }
}