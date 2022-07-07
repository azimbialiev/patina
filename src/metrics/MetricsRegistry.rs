use crate::connection::rx_connection_handler::{ RxConnectionHandlerMetrics};
use crate::connection::tx_connection_handler::{ TxConnectionHandlerMetrics};

#[derive(serde::Serialize)]
pub struct ServiceMetricRegistry<'a> {
    pub(crate) rx_connection_handler: &'a RxConnectionHandlerMetrics,
    pub(crate) tx_connection_handler: &'a TxConnectionHandlerMetrics,
}
