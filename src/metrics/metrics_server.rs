use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use warp::Filter;

use crate::{PacketHandler, RxConnectionHandler, ServiceMetricRegistry, TxConnectionHandler};

pub async fn start_metrics_server(rx_connection_handler: Arc<RxConnectionHandler>, tx_connection_handler: Arc<TxConnectionHandler>, packet_handler: Arc<PacketHandler>){
    info!("Prometheus metrics exposed on 127.0.0.1:9000");

    let routes = warp::get()
        .and(warp::path("metrics"))
        .map(move || {
            let registry = &ServiceMetricRegistry {
                rx_connection_handler: &rx_connection_handler.metrics,
                tx_connection_handler: &tx_connection_handler.metrics,
                packet_handler: &packet_handler.metrics,
                mqtt_decoder: &rx_connection_handler.decoder.metrics
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