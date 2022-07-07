use crate::connection::rx_connection_handler::{RxConnectionHandlerMetrics};
use crate::connection::tx_connection_handler::{TxConnectionHandlerMetrics};
use crate::broker::packet_handler::{PacketHandlerMetrics};

#[derive(serde::Serialize)]
pub struct ServiceMetricRegistry<'a> {
    pub(crate) rx_connection_handler: &'a RxConnectionHandlerMetrics,
    pub(crate) tx_connection_handler: &'a TxConnectionHandlerMetrics,
    pub(crate) packet_handler: &'a PacketHandlerMetrics,
}