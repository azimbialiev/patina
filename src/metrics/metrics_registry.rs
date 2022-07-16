use crate::broker::handler::connect_handler::ConnectHandlerMetrics;
use crate::broker::handler::disconnect_handler::DisconnectHandlerMetrics;
use crate::broker::handler::pingreq_handler::PingreqHandlerMetrics;
use crate::broker::handler::publish_handler::PublishHandlerMetrics;
use crate::broker::handler::pubrec_handler::PubrecHandlerMetrics;
use crate::broker::handler::pubrel_handler::PubrelHandlerMetrics;
use crate::broker::handler::subscribe_handler::SubscribeHandlerMetrics;
use crate::broker::handler::unsubscribe_handler::UnsubscribeHandlerMetrics;
use crate::broker::packet_dispatcher::{*};
use crate::connection::rx_connection_handler::RxClientHandlerMetrics;
use crate::connection::tx_connection_handler::TxClientHandlerMetrics;
use crate::serdes::deserializer::fixed_header_decoder::FixedHeaderDecoderMetrics;
use crate::serdes::deserializer::payload_decoder::PayloadDecoderMetrics;
use crate::serdes::deserializer::variable_header_decoder::VariableHeaderDecoderMetrics;
use crate::serdes::mqtt_decoder::MqttDecoderMetrics;
use crate::serdes::mqtt_encoder::MqttEncoderMetrics;
use crate::session::client_handler::ClientHandlerMetrics;
//use crate::session::session_handler::SessionHandlerMetrics;
use crate::topic::topic_handler::TopicHandlerMetrics;

#[derive(Clone)]
#[derive(serde::Serialize)]
pub struct ServiceMetricRegistry<'a> {
    pub(crate) rx_client_handler: &'a RxClientHandlerMetrics,
    pub(crate) tx_client_handler: &'a TxClientHandlerMetrics,
    pub(crate) packet_dispatcher: &'a PacketDispatcherMetrics,
    pub(crate) mqtt_decoder: &'a MqttDecoderMetrics,
    pub(crate) fixed_header_decoder: &'a FixedHeaderDecoderMetrics,
    pub(crate) variable_header_decoder: &'a VariableHeaderDecoderMetrics,
    pub(crate) payload_decoder: &'a PayloadDecoderMetrics,
    pub(crate) mqtt_encoder: &'a MqttEncoderMetrics,
    pub(crate) client_handler: &'a ClientHandlerMetrics,
    pub(crate) topic_handler: &'a TopicHandlerMetrics,
    pub(crate) connect_handler: &'a ConnectHandlerMetrics,
    pub(crate) disconnect_handler: &'a DisconnectHandlerMetrics,
    pub(crate) pingreq_handler: &'a PingreqHandlerMetrics,
    pub(crate) publish_handler: &'a PublishHandlerMetrics,
    pub(crate) pubrec_handler: &'a PubrecHandlerMetrics,
    pub(crate) pubrel_handler: &'a PubrelHandlerMetrics,
    pub(crate) subscribe_handler: &'a SubscribeHandlerMetrics,
    pub(crate) unsubscribe_handler: &'a UnsubscribeHandlerMetrics
}
