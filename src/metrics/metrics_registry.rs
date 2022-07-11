use crate::broker::packet_handler::PacketHandlerMetrics;
use crate::session::client_handler::{ClientHandlerMetrics};
use crate::connection::rx_connection_handler::RxClientHandlerMetrics;
use crate::connection::tx_connection_handler::TxClientHandlerMetrics;
use crate::serdes::mqtt_decoder::MqttDecoderMetrics;
use crate::serdes::mqtt_encoder::MqttEncoderMetrics;
//use crate::session::session_handler::SessionHandlerMetrics;
use crate::topic::topic_handler::TopicHandlerMetrics;

#[derive(Clone)]
#[derive(serde::Serialize)]
pub struct ServiceMetricRegistry<'a> {
    pub(crate) rx_client_handler: &'a RxClientHandlerMetrics,
    pub(crate) tx_client_handler: &'a TxClientHandlerMetrics,
    pub(crate) packet_handler: &'a PacketHandlerMetrics,
    pub(crate) mqtt_decoder: &'a MqttDecoderMetrics,
    pub(crate) mqtt_encoder: &'a MqttEncoderMetrics,
    pub(crate) client_handler: &'a ClientHandlerMetrics,
    //pub(crate) session_handler: &'a ClientHandlerMetrics,
    pub(crate) topic_handler: &'a TopicHandlerMetrics
}
