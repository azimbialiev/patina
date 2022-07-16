use bitreader::BitReader;
use log::{debug, error, trace};
use metered::{*};
use crate::model::variable_header::Property;
use crate::serdes::deserializer::error::{DecodeError, DecodeResult, ReadError};
use crate::serdes::r#trait::decoder::Decoder;

#[derive(Default, Debug)]
pub struct PropertyDecoder {
    pub(crate) metrics: PropertyDecoderMetrics,

}


impl PropertyDecoder {
    fn read_property_length(&self, reader: &mut BitReader) -> DecodeResult<u64> {
        trace!("PropertyDecoder::read_property_length");

        let property_length = match self.read_variable_byte_integer(reader) {
            Ok(result) => { result }
            Err(err) => {
                error!("Can't decode Property Length: {:?}", err);
                return Err(DecodeError::PropertyLength { cause: err.cause() });
            }
        };
        trace!("Extracted Property Length: {:?}", property_length);
        return Ok(property_length);
    }

    fn read_property(&self, identifier: u64, reader: &mut BitReader) -> DecodeResult<Option<Property>> {
        let map_error = |err: ReadError| {
            error!("Can't decode property: {:?}", err);
            return Err(DecodeError::Property { cause: err });
        };
        return match identifier {
            0 => {
                let value = self.read_variable_byte_integer(reader)?;
                Ok(Some(Property::SubscriptionIdentifier(value)))
            }
            1 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::PayloadFormatIndicator(value)))
            }
            2 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MessageExpiryInterval(value)))
            }
            3 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ContentType(value)))
            }
            8 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ResponseTopic(value)))
            }
            9 => {
                let value = self.read_binary_data(reader)?;
                Ok(Some(Property::CorrelationData(value)))
            }
            11 => {
                let value = self.read_variable_byte_integer(reader)?;
                Ok(Some(Property::SubscriptionIdentifier(value)))
            }
            17 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SessionExpiryInterval(value)))
            }
            18 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::AssignedClientIdentifier(value)))
            }
            19 => {
                let value = match self.read_u8(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::ServerKeepAlive(value)))
            }
            21 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::AuthenticationMethod(value)))
            }
            22 => {
                let value = self.read_binary_data(reader)?;
                Ok(Some(Property::AuthenticationData(value)))
            }
            23 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RequestProblemInformation(value)))
            }
            24 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::WillDelayInterval(value)))
            }
            25 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RequestResponseInformation(value)))
            }
            26 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ResponseInformation(value)))
            }
            28 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ServerReference(value)))
            }
            31 => {
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::ReasonString(value)))
            }
            33 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::ReceiveMaximum(value)))
            }
            34 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::TopicAliasMaximum(value)))
            }
            35 => {
                let value = match self.read_u16(8 * 2, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::TopicAlias(value)))
            }
            36 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MaximumQoS(value)))
            }
            37 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::RetainAvailable(value)))
            }
            38 => {
                let key = self.read_utf8_string(reader)?;
                let value = self.read_utf8_string(reader)?;
                Ok(Some(Property::UserProperty(key, value)))
            }
            39 => {
                let value = match self.read_u32(8 * 4, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::MaximumPacketSize(value)))
            }
            40 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::WildcardSubscriptionAvailable(value)))
            }
            41 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SubscriptionIdentifierAvailable(value)))
            }
            42 => {
                let value = match self.read_u8(8, reader) {
                    Ok(result) => { result }
                    Err(err) => { return map_error(err); }
                };
                Ok(Some(Property::SharedSubscriptionAvailable(value)))
            }
            unknown_value => {
                error!("Can't build Property. Unknown identifier {:?}", unknown_value);
                Err(DecodeError::UnknownProperty { cause: ReadError::InvalidData })
            }
        };
    }
}

#[metered(registry = PropertyDecoderMetrics)]
impl Decoder<Vec<Property>> for PropertyDecoder {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    fn decode(&self, reader: &mut BitReader) -> DecodeResult<Vec<Property>> {
        debug!("PropertyDecoder::decode");
        let mut properties: Vec<Property> = Vec::new();
        let start_position = reader.position();
        let mut properties_byte_size = self.read_property_length(reader)?;
        //Property Length in bytes
        trace!("Properties Byte Size: {:?}", properties_byte_size);

        while properties_byte_size > 0 {
            let properties_start = reader.position();
            let identifier = self.read_variable_byte_integer(reader)?;
            trace!("Property Identifier: {:?}", identifier);
            let property = self.read_property(identifier, reader)?;
            trace!("Extracted Property: {:?}", property);
            if property.is_some() {
                properties.push(property.unwrap());
            }
            //I need to check how many bytes Property Identifier and their values consumed from stream
            let consumed_bytes = (reader.position() - properties_start) / 8;
            properties_byte_size = properties_byte_size - consumed_bytes;
            trace!("Consumed bytes: {:?}. Remaining properties bytes: {:?}", consumed_bytes, properties_byte_size);
        }
        trace!("Properties consumed {:?} bytes from stream", (reader.position() - start_position) / 8);

        return Ok(properties);
    }
}