use bitreader::{BitReader, BitReaderError};
use log::{debug, error, trace};

use crate::model::fixed_header::ControlPacketType;
use crate::model::payload::Payload;
use crate::model::qos_level::QoSLevel;
use crate::model::topic::{RetainHandling, TopicFilter};
use crate::model::variable_header::{Property, VariableHeader};
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::deserializer::property_decoder::PropertyDecoder;
use crate::serdes::r#trait::decoder::Decoder;

pub struct PayloadDecoder {
    packet_type: ControlPacketType,
    variable_header: Option<VariableHeader>,
}

impl PayloadDecoder {
    pub fn variable_header(&self) -> &VariableHeader {
        self.variable_header.as_ref().unwrap()
    }
}


impl PayloadDecoder {
    pub fn new(packet_type: ControlPacketType, variable_header: Option<VariableHeader>) -> Self {
        PayloadDecoder { packet_type, variable_header }
    }

    fn read_topic_path(&self, reader: &mut BitReader) -> DecodeResult<String> {
        trace!("PayloadDecoder::read_topic_path");
        let topic_path = match self.read_utf8_string(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Topic Filter: {:?}", err);
                return Err(DecodeError::TopicFilter { cause: err.cause() });
            }
        };
        Ok(topic_path)
    }

    fn read_topic_filter(&self, reader: &mut BitReader) -> DecodeResult<TopicFilter> {
        trace!("PayloadDecoder::read_topic_filter");
        let topic_filter = self.read_topic_path(reader)?;
        trace!("Extracted Topic Filter: {:?}", topic_filter);
        let reserved_bits = match self.read_booleans(2, reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Reserved Bits: {:?}", err);
                return Err(DecodeError::ReservedFlag { cause: err });
            }
        };
        trace!("Extracted Reserved Bits: {:?}", reserved_bits);

        let retain_handling = match self.read_u8(2, reader) {
            Ok(retain_handling) => {
                match RetainHandling::from_u8(retain_handling) {
                    Some(retain_handling) => { retain_handling }
                    None => {
                        error!("Can't decode RetainHandling from value: {:?}", retain_handling);
                        return Err(DecodeError::RetainHandling { cause: ReadError::ExceededMaxValue { current: retain_handling as u64, max: 2 } });
                    }
                }
            }
            Err(err) => {
                error!("Can't read RetainHandling from: {:?}", err);
                return Err(DecodeError::RetainHandling { cause: err });
            }
        };
        trace!("Extracted Retain Handling: {:?}", retain_handling);

        let retain_as_published = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read Retain As Published: {:?}", err);
                return Err(DecodeError::RetainAsPublished { cause: err });
            }
        };
        trace!("Extracted Retain As Published: {:?}", retain_as_published);

        let no_local = match self.read_bool(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't read No Local: {:?}", err);
                return Err(DecodeError::NoLocal { cause: err });
            }
        };
        trace!("Extracted No Local: {:?}", no_local);

        let maximum_qos = match self.read_u8(2, reader) {
            Ok(qos_level) => {
                match QoSLevel::from_u8(qos_level) {
                    Some(qos_level) => { qos_level }
                    None => {
                        error!("Can't decode MaximumQoS from value: {:?}", qos_level);
                        return Err(DecodeError::MaximumQoS { cause: ReadError::ExceededMaxValue { current: qos_level as u64, max: 2 } });
                    }
                }
            }
            Err(err) => {
                error!("Can't read MaximumQoS: {:?}", err);
                return Err(DecodeError::MaximumQoS { cause: err });
            }
        };
        trace!("Extracted Maximum QoS: {:?}", maximum_qos);

        Ok(TopicFilter::from_subscribe(topic_filter, maximum_qos, no_local, retain_as_published, retain_handling, reserved_bits))
    }
}

impl Decoder<Option<Payload>> for PayloadDecoder {
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Option<Payload>> {
        debug!("PayloadDecoder::decode");
        return Ok(match self.packet_type {
            ControlPacketType::RESERVED => { None }
            ControlPacketType::CONNECT => {
                let client_id = self.read_utf8_string(reader)?;
                trace!("Extracted Client ID: {:?}", client_id);

                let connect_flags = self.variable_header().connect_flags();

                let mut will_properties: Option<Vec<Property>> = None;
                let mut will_topic: Option<String> = None;
                let mut will_payload: Option<Vec<u8>> = None;

                if connect_flags.will_flag() {
                    let property_decoder = PropertyDecoder::new();
                    will_properties = Option::from(property_decoder.decode(reader)?);
                    trace!("Extracted Will Properties: {:?}", will_properties);

                    will_topic = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Will Topic: {:?}", will_topic);

                    will_payload = Option::from(self.read_binary_data(reader)?);
                    trace!("Extracted Will Payload: {:?}", will_payload);
                }

                let mut username: Option<String> = None;
                if connect_flags.username_flag() {
                    username = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Username: {:?}", username);
                }

                let mut password: Option<String> = None;
                if connect_flags.password_flag() {
                    password = Option::from(self.read_utf8_string(reader)?);
                    trace!("Extracted Password: {:?}", password);
                }

                Option::from(Payload::from_connect(Some(client_id), will_properties, will_topic, will_payload, username, password))
            }
            ControlPacketType::CONNACK => { None }
            ControlPacketType::PUBLISH => {
                let mut data = Vec::with_capacity((reader.remaining() / 8) as usize);
                while data.len() != data.capacity() {
                    data.push(
                        match reader.read_u8(8) {
                            Ok(result) => { result }
                            Err(err) => {
                                error!("Can't read payload: {:?}", err);
                                return match err {
                                    BitReaderError::NotEnoughData {
                                        position,
                                        length,
                                        requested, } => {
                                        Err(DecodeError::Payload { cause: ReadError::NotEnoughData { position, length, requested } })
                                    }
                                    BitReaderError::TooManyBitsForType {
                                        position,
                                        requested,
                                        allowed, } => {
                                        Err(DecodeError::Payload { cause: ReadError::TooManyBitsForType { position, requested, allowed } })
                                    }
                                };
                            }
                        })
                }
                Option::from(Payload::from_publish(Option::from(data)))
            }
            ControlPacketType::PUBACK => { None }
            ControlPacketType::PUBREC => { None }
            ControlPacketType::PUBREL => { None }
            ControlPacketType::PUBCOMP => { None }
            ControlPacketType::SUBSCRIBE => {
                let mut topic_filters = Vec::new();
                while reader.remaining() != 0 {
                    let topic_filter = self.read_topic_filter(reader)?;
                    topic_filters.push(topic_filter);
                }
                Option::from(Payload::from_sub_unsub(topic_filters))
            }
            ControlPacketType::SUBACK => { None }
            ControlPacketType::UNSUBSCRIBE => {
                let mut topic_filters = Vec::new();
                while reader.remaining() != 0 {
                    let topic_path = self.read_topic_path(reader)?;
                    topic_filters.push(TopicFilter::from_unsubscribe(topic_path));
                }
                Option::from(Payload::from_sub_unsub(topic_filters))
            }
            ControlPacketType::UNSUBACK => { None }
            ControlPacketType::PINGREQ => { None }
            ControlPacketType::PINGRESP => { None }
            ControlPacketType::DISCONNECT => { None }
            ControlPacketType::AUTH => { None }
        });
    }
}

