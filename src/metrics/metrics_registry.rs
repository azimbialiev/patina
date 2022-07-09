use crate::broker::packet_handler::PacketHandlerMetrics;
use crate::connection::rx_connection_handler::RxClientHandlerMetrics;
use crate::connection::tx_connection_handler::TxClientHandlerMetrics;
use crate::serdes::mqtt_decoder::MqttDecoderMetrics;
use crate::serdes::mqtt_encoder::MqttEncoderMetrics;

#[derive(Clone)]
#[derive(serde::Serialize)]
pub struct ServiceMetricRegistry<'a> {
    pub(crate) rx_client_handler: &'a RxClientHandlerMetrics,
    pub(crate) tx_client_handler: &'a TxClientHandlerMetrics,
    pub(crate) packet_handler: &'a PacketHandlerMetrics,
    pub(crate) mqtt_decoder: &'a MqttDecoderMetrics,
    pub(crate) mqtt_encoder: &'a MqttEncoderMetrics
}
