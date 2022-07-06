use core::fmt;
use std::io::ErrorKind;

use bitreader::BitReader;
use bytes::BytesMut;
use log::{debug, error, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::MutexGuard;
use tokio::time::Instant;

use crate::serdes::decoder::{DecodeError, Decoder, DecodeResult, FixedHeaderDecoder, PayloadDecoder, ReadError, VariableHeaderDecoder};
use crate::serdes::encoder::{Encoder, EncodeResult, FixedHeaderEncoder, LengthCalculator, OptEncoder, PayloadEncoder, VariableHeaderEncoder};
use crate::serdes::mqtt::ControlPacket;

pub type WriteResult = Result<(), WriteError>; //TODO Needs better errors

#[derive(Debug, PartialEq, Clone)]
pub enum WriteError {
    ConnectionTimedOut,
    EncodeError,
    SendError,
    FlushError,
}

impl fmt::Display for WriteError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //self.description().fmt(fmt)
        match *self {
            WriteError::ConnectionTimedOut => write!(fmt, "WriteError::ConnectionTimedOut"),
            WriteError::EncodeError => write!(fmt, "WriteError::EncodeError"),
            WriteError::SendError => write!(fmt, "WriteError::SendError"),
            WriteError::FlushError => write!(fmt, "WriteError::FlushError"),
        }
    }
}

pub async fn write_buffer(buffer: &BytesMut, stream: &mut OwnedWriteHalf) -> WriteResult {
    let now = Instant::now();
    debug!("MQTTConnection::write");
    trace!("Buffer Length: {:?}", buffer.len());
    for i in buffer.clone() {
        trace!("Going to write: {:#04X?}", i );
    }

    match stream.write(buffer).await {
        Ok(result) => {
            trace!("{:?} bytes written to stream", result);
        }
        Err(e) => {
            error!("Can't send packet: {:?}", e);
            return Err(WriteError::SendError);
        }
    };
    //Ok(())
    match stream.flush().await {
        Ok(_) => { Ok(()) }
        Err(e) => {
            error!("Can't flush buffered writer: {:?}", e);
            return Err(WriteError::FlushError);
        }
    }
}

pub fn encode_packet(packet: &ControlPacket) -> EncodeResult<BytesMut> {
    let now = Instant::now();
    debug!("Connection::encode_packet");
    trace!("Encoding packet: {:?} - {:?}", packet.fixed_header().packet_type(), packet);
    let mut calculated_remaining_length = 0;

    let mut payload_encoder = PayloadEncoder::new(packet.fixed_header().packet_type());
    match packet.payload_opt() {
        None => {}
        Some(payload) => {
            let remaining_length = payload_encoder.calculate_length(payload) as u64;
            trace!("Payload Length: {:?}", remaining_length);
            calculated_remaining_length = calculated_remaining_length + remaining_length;
        }
    }

    let mut variable_header_encoder = VariableHeaderEncoder::new(packet.fixed_header().packet_type());
    match packet.variable_header_opt() {
        None => {}
        Some(variable_header) => {
            let remaining_length = variable_header_encoder.calculate_length(variable_header) as u64;
            trace!("Variable Header Length: {:?}", remaining_length);
            calculated_remaining_length = calculated_remaining_length + remaining_length;
        }
    }

    let mut fixed_header_encoder = FixedHeaderEncoder::new();
    let fixed_header_length = fixed_header_encoder.calculate_length(&(packet.fixed_header(), calculated_remaining_length));
    trace!("Fixed Header Length: {:?}", fixed_header_length);
    debug!("Control Packet Remaining Length: {:?}", calculated_remaining_length);
    let mut buffer = BytesMut::with_capacity(fixed_header_length + calculated_remaining_length as usize);
    fixed_header_encoder.encode(&(packet.fixed_header(), calculated_remaining_length), &mut buffer)?;
    variable_header_encoder.encode_opt(packet.variable_header_opt(), &mut buffer)?;
    payload_encoder.encode_opt(packet.payload_opt(), &mut buffer).expect("panic encode_opt");
    Ok(buffer)
}

pub async fn read_packet(stream: MutexGuard<'_, OwnedReadHalf>)
                         -> DecodeResult<ControlPacket>
{
    let now = Instant::now();
    debug!("MQTTConnection::read_packet");
    return match decode_packet(stream).await {
        Ok(result) => {
            Ok(result)
        }
        Err(err) => {
            Err(err)
        }
    };
    // return match timeout(Duration::from_secs(5), read_packet_from_stream(stream)).await {
    //     Ok(result) => { result }
    //     Err(err) => {
    //         error!("Can't handle connection: {:?}", err);
    //         Err(DecodeError::ConnectionTimedOut{cause: ReadError::IOError})
    //     }
    // };
}


async fn decode_packet(mut stream: MutexGuard<'_, OwnedReadHalf>) -> DecodeResult<ControlPacket> {
    let now = Instant::now();
    debug!("MQTTConnection::decode_packet");
    //let mut buffer = BytesMut::with_capacity(3);
    //let mut buf_reader = BufReader::new(stream);

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


