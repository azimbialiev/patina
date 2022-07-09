use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use warp::Filter;

use crate::{Broker, RxConnectionHandler, ServiceMetricRegistry, TxConnectionHandler};

pub async fn start_metrics_server(rx_connection_handler: Arc<RxConnectionHandler>, tx_connection_handler: Arc<TxConnectionHandler>, broker: Arc<Broker>){
    info!("Prometheus metrics exposed on 127.0.0.1:9000");

    let routes = warp::get()
        .and(warp::path("metrics"))
        .map(move || {
            let registry = &ServiceMetricRegistry {
                rx_client_handler: &rx_connection_handler.client_handler.metrics,
                tx_client_handler: &tx_connection_handler.client_handler.metrics,
                packet_handler: &broker.packet_handler.metrics,
                mqtt_decoder: &rx_connection_handler.client_handler.decoder.metrics,
                mqtt_encoder: &tx_connection_handler.client_handler.encoder.metrics
            };
            let globals = HashMap::new();
            serde_prometheus::to_string(
                &registry,
                Some("patina"),
                globals,
            ).unwrap()
        });

    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}