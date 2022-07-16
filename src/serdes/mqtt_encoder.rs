use std::ops::Deref;
use std::sync::Arc;

use bytes::BytesMut;
use log::{debug, trace};
use metered::{*};
use nameof::name_of_type;
use serde::Serializer;

use crate::model::control_packet::ControlPacket;
use crate::serdes::r#trait::encoder::{Encoder, LengthCalculator, OptEncoder};
use crate::serdes::serializer::error::EncodeResult;
use crate::serdes::serializer::fixed_header_encoder::FixedHeaderEncoder;
use crate::serdes::serializer::payload_encoder::PayloadEncoder;
use crate::serdes::serializer::variable_header_encoder::VariableHeaderEncoder;

#[derive(Default, Clone, Debug)]
pub struct MqttEncoder(Arc<MqttEncoderImpl>);

impl Deref for MqttEncoder {
    type Target = MqttEncoderImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl serde::Serialize for MqttEncoder{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        return self.deref().serialize(serializer);
    }
}


#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct MqttEncoderImpl {
    pub(crate) metrics: MqttEncoderMetrics,

}

#[metered(registry = MqttEncoderMetrics)]
impl MqttEncoderImpl {


    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn encode_packet(&self, packet: &Arc<ControlPacket>) -> EncodeResult<BytesMut> {
        debug!("{}::encode_packet", name_of_type!(MqttEncoder));
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

}