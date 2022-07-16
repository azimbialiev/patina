use bitreader::BitReader;
use log::{debug, error, trace};
use metered::{*};
use crate::model::fixed_header::{ControlPacketType, FixedHeader};
use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;
use crate::model::variable_header::{ConnectFlags, VariableHeader};
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::deserializer::property_decoder::PropertyDecoder;
use crate::serdes::r#trait::decoder::Decoder;

#[derive(Default, Debug)]
pub struct VariableHeaderDecoder {
    pub(crate) metrics: VariableHeaderDecoderMetrics,
    pub(crate) property_decoder: PropertyDecoder,

}

#[metered(registry = VariableHeaderDecoderMetrics)]
impl VariableHeaderDecoder {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn decode_with_header(&self, fixed_header: &FixedHeader, reader: &mut BitReader) -> DecodeResult<Option<VariableHeader>> {
        debug!("VariableHeaderDecoder::decode");
        return Ok(match fixed_header.packet_type() {
            ControlPacketType::RESERVED => { None }
            ControlPacketType::CONNECT => {
                let start_position = reader.position();

                let protocol_name = self.read_protocol_name(reader)?;
                let protocol_version = self.read_protocol_version(reader)?;
                let connect_flags = self.read_connect_flags(reader)?;
                let keep_alive = self.read_keep_alive(reader)?;
                let properties = self.property_decoder.decode(reader)?;
                trace!("Variable Header consumed {:?} bytes from stream", (reader.position() - start_position) / 8);


                Some(VariableHeader::from_connect(Some(protocol_name), Some(protocol_version), Some(connect_flags), Some(keep_alive), properties))
            }
            ControlPacketType::CONNACK => { None }
            ControlPacketType::PUBLISH => {
                let topic_name = match self.read_utf8_string(reader) {
                    Ok(result) => { result }
                    Err(err) => {
                        error!("Can't decode Topic Name: {:?}", err);
                        return Err(DecodeError::TopicName { cause: err.cause() });
                    }
                };
                trace!("Extracted Topic Name: {:?}", topic_name);
                let mut packet_identifier = None;

                match fixed_header.qos_level() {
                    QoSLevel::AtMostOnce => {}
                    _ => {
                        packet_identifier = Some(self.read_packet_identifier(reader)?);
                        trace!("Extracted Packet Identifier: {:?}", packet_identifier);
                    }
                }

                let properties = self.property_decoder.decode(reader)?;
                Some(VariableHeader::from_publish(packet_identifier, Some(topic_name), properties))
            }
            ControlPacketType::PUBACK => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = self.property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBREC => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = self.property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBREL => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = self.property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::PUBCOMP => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let mut reason_code = None;
                if reader.remaining() > 8 {
                    reason_code = Some(self.read_reason_code(reader)?);
                }
                let mut properties = vec![];
                if reader.remaining() > 8 {
                    properties = self.property_decoder.decode(reader)?;
                }
                Some(VariableHeader::from_pub_ack_rel_comp(Some(packet_identifier), reason_code, vec![]))
            }
            ControlPacketType::SUBSCRIBE => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let properties = self.property_decoder.decode(reader)?;
                Some(VariableHeader::from_sub_unsub(Some(packet_identifier), properties))
            }
            ControlPacketType::SUBACK => { None }
            ControlPacketType::UNSUBSCRIBE => {
                let packet_identifier = self.read_packet_identifier(reader)?;
                let properties = self.property_decoder.decode(reader)?;
                Some(VariableHeader::from_sub_unsub(Some(packet_identifier), properties))
            }
            ControlPacketType::UNSUBACK => { None }
            ControlPacketType::PINGREQ => { None }
            ControlPacketType::PINGRESP => { None }
            ControlPacketType::DISCONNECT => {
                let reason_code = self.read_reason_code(reader)?;
                let properties = self.property_decoder.decode(reader)?;
                Some(VariableHeader::from_disconnect(reason_code, properties))
            }
            ControlPacketType::AUTH => { None }
        });
    }

    fn read_protocol_name(&self, reader: &mut BitReader) -> DecodeResult<String> {
        trace!("VariableHeaderDecoder::read_protocol_name");
        let protocol_name = self.read_utf8_string(reader)?;
        trace!("Extracted Protocol Name: {:?}", protocol_name);
        return Ok(protocol_name);
    }

    fn read_protocol_version(&self, reader: &mut BitReader) -> DecodeResult<u8> {
        trace!("VariableHeaderDecoder::read_protocol_version");
        let protocol_version = match self.read_u8(8, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Protocol Version: {:?}", err);
                return Err(DecodeError::ProtocolVersion { cause: err });
            }
        };
        trace!("Extracted Protocol Version: {:?}", protocol_version);
        return Ok(protocol_version);
    }

    fn read_connect_flags(&self, reader: &mut BitReader) -> DecodeResult<ConnectFlags> {
        trace!("VariableHeaderDecoder::read_connect_flags");

        let username_flag = self.read_username_flag(reader)?;
        let password_flag = self.read_password_flag(reader)?;
        let will_retain_flag = self.read_will_retain_flag(reader)?;
        let will_qos = self.read_will_qos_flags(reader)?;
        let will_flag = self.read_will_flag(reader)?;
        let clean_start_flag = self.read_clean_start_flag(reader)?;
        let reserved_flag = self.read_reserved_flag(reader)?;
        return Ok(ConnectFlags::new(
            username_flag,
            password_flag,
            will_retain_flag,
            will_qos,
            will_flag,
            clean_start_flag,
            reserved_flag,
        ));
    }

    fn read_username_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_username_flag");

        let username_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Username Flag: {:?}", err);
                return Err(DecodeError::UsernameFlag { cause: err });
            }
        };
        trace!("Extracted User Name Flag: {:?}", username_flag);
        return Ok(username_flag);
    }

    fn read_password_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_password_flag");

        let password_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Password Flag: {:?}", err);
                return Err(DecodeError::PasswordFlag { cause: err });
            }
        };
        trace!("Extracted Password Flag: {:?}", password_flag);
        return Ok(password_flag);
    }

    fn read_will_retain_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_will_retain_flag");

        let will_retain_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will Retain Flag: {:?}", err);
                return Err(DecodeError::UsernameFlag { cause: err });
            }
        };
        trace!("Extracted Will Retain Flag: {:?}", will_retain_flag);
        return Ok(will_retain_flag);
    }

    fn read_will_qos_flags(&self, reader: &mut BitReader) -> DecodeResult<QoSLevel> {
        trace!("VariableHeaderDecoder::read_will_qos_flags");

        let will_qos_raw = match self.read_u8(2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will QoS Flag: {:?}", err);
                return Err(DecodeError::WillQoSFlag { cause: err });
            }
        };
        let will_qos = QoSLevel::from_u8(will_qos_raw);
        return match will_qos {
            None => {
                error!("Invalid Will QoS: {:?}", will_qos_raw);
                Err(DecodeError::WillQoSFlag { cause: ReadError::InvalidData })
            }
            Some(will_qos) => {
                trace!("Extracted Will QoS: {:?}", will_qos);
                Ok(will_qos)
            }
        };
    }

    fn read_clean_start_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_clean_start_flag");

        let clean_start = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Clean Start Flag: {:?}", err);
                return Err(DecodeError::CleanStartFlag { cause: err });
            }
        };
        trace!("Extracted Clean Start Flag: {:?}", clean_start);
        return Ok(clean_start);
    }


    fn read_will_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_will_flag");

        let will_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Will Flag: {:?}", err);
                return Err(DecodeError::WillFlag { cause: err });
            }
        };
        trace!("Extracted Will Flag: {:?}", will_flag);
        return Ok(will_flag);
    }

    fn read_reserved_flag(&self, reader: &mut BitReader) -> DecodeResult<bool> {
        trace!("VariableHeaderDecoder::read_reserved_flag");

        let reserved_flag = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Reserved Flag: {:?}", err);
                return Err(DecodeError::ReservedFlag { cause: err });
            }
        };
        trace!("Extracted Reserved Flag: {:?}", reserved_flag);
        return Ok(reserved_flag);
    }

    fn read_keep_alive(&self, reader: &mut BitReader) -> DecodeResult<u16> {
        trace!("VariableHeaderDecoder::read_keep_alive");

        let keep_alive = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Keep Alive: {:?}", err);
                return Err(DecodeError::KeepAlive { cause: err });
            }
        };
        //time interval measured in seconds
        trace!("Extracted Keep Alive: {:?}", keep_alive);
        return Ok(keep_alive);
    }
    fn read_packet_identifier(&self, reader: &mut BitReader) -> DecodeResult<u16> {
        let packet_identifier = match self.read_u16(8 * 2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Packet Identifier: {:?}", err);
                return Err(DecodeError::PacketIdentifier { cause: err });
            }
        };
        Ok(packet_identifier)
    }

    fn read_reason_code(&self, reader: &mut BitReader) -> DecodeResult<ReasonCode> {
        return match self.read_u8(8, reader) {
            Ok(result) => {
                match ReasonCode::from_u8(result) {
                    Some(reason_code) => { Ok(reason_code) }
                    None => {
                        error!("Can't decode ReasonCode from value: {:?}", result);
                        Err(DecodeError::ReasonCode { cause: ReadError::InvalidData })
                    }
                }
            }
            Err(err) => {
                error!("Can't decode Packet Identifier: {:?}", err);
                return Err(DecodeError::ReasonCode { cause: err });
            }
        };
    }
}

impl Decoder<Option<VariableHeader>> for VariableHeaderDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Option<VariableHeader>> {
        unimplemented!()
    }
}