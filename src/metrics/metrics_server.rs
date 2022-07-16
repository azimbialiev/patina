use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use warp::Filter;

use crate::{Broker, RxConnectionHandler, ServiceMetricRegistry, TxConnectionHandler};

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
pub async fn start_metrics_server(
    rx_connection_handler: Arc<RxConnectionHandler>,
    tx_connection_handler: Arc<TxConnectionHandler>,
    broker: Arc<Broker>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Prometheus metrics exposed on 127.0.0.1:9000");

    let routes = warp::get()
        .and(warp::path("metrics"))
        .map(move || {
            let registry = &ServiceMetricRegistry {
                rx_client_handler: &rx_connection_handler.rx_client_handler.metrics,
                tx_client_handler: &tx_connection_handler.tx_client_handler.metrics,
                packet_dispatcher: &broker.packet_dispatcher.metrics,
                mqtt_decoder: &rx_connection_handler.rx_client_handler.decoder.metrics,
                fixed_header_decoder: &rx_connection_handler.rx_client_handler.decoder.fixed_header_decoder.metrics,
                variable_header_decoder: &rx_connection_handler.rx_client_handler.decoder.variable_header_decoder.metrics,
                payload_decoder: &rx_connection_handler.rx_client_handler.decoder.payload_decoder.metrics,
                mqtt_encoder: &tx_connection_handler.encoder.metrics,
                client_handler: &broker.packet_dispatcher.client_handler.metrics,
                topic_handler: &broker.packet_dispatcher.topic_handler.metrics,
                connect_handler: &broker.packet_dispatcher.connect_handler.metrics,
                disconnect_handler: &broker.packet_dispatcher.disconnect_handler.metrics,
                pingreq_handler: &broker.packet_dispatcher.pingreq_handler.metrics,
                publish_handler:&broker.packet_dispatcher.publish_handler.metrics,
                pubrec_handler: &broker.packet_dispatcher.pubrec_handler.metrics,
                pubrel_handler: &broker.packet_dispatcher.pubrel_handler.metrics,
                subscribe_handler: &broker.packet_dispatcher.subscribe_handler.metrics,
                unsubscribe_handler: &broker.packet_dispatcher.unsubscribe_handler.metrics
            };
            let globals = HashMap::new();
            serde_prometheus::to_string(
                &registry,
                Some("patina"),
                globals,
            ).unwrap()
        });

    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
    Ok(())
}