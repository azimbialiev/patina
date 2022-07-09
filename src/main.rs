extern crate core;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

use log::{info, warn};
use log4rs;
use tokio::sync::{mpsc, Mutex};

use crate::broker::broker::Broker;
use crate::connection::rx_connection_handler::RxConnectionHandler;
use crate::connection::tx_connection_handler::TxConnectionHandler;
use crate::metrics::metrics_registry::ServiceMetricRegistry;
use crate::session::client_handler::ClientHandler;
use crate::topic::topic_handler::TopicHandler;

mod tests;
mod topic;
mod connection;
mod serdes;
mod broker;
mod session;
mod metrics;
mod model;

pub fn init_logging() {
    log4rs::init_file("config/log4rs.yaml", Default::default());
}


#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    info!("MQTT SERVER");
    let (listener2broker_tx, listener2broker_rx) = mpsc::channel(10000);
    let (broker2listener_tx, broker2listener_rx) = mpsc::channel(10000);
    let stream_repository = Arc::new(DashMap::new());

    let packet_handler = Arc::new(Broker::default());
    let packet_handler_ = packet_handler.clone();
    tokio::spawn(async move {
        info!("Spawned Broker thread");
        packet_handler_.handle_packets(
            listener2broker_rx,
            broker2listener_tx
        ).await;
        warn!("Broker thread going to die");
    });

    let stream_repository_ = stream_repository.clone();
    let tx_connection_handler = Arc::new(TxConnectionHandler::default());
    let tx_connection_handler_ = tx_connection_handler.clone();
    let listener2broker_tx_ = listener2broker_tx.clone();
    tokio::spawn(async move {
        info!("Spawned TxConnectionHandler thread");
        tx_connection_handler_.handle_outgoing_connections(broker2listener_rx, listener2broker_tx_, stream_repository_).await;
        warn!("TxConnectionHandler thread going to die");
    });

    let stream_repository_ = stream_repository.clone();
    let rx_connection_handler = Arc::new(RxConnectionHandler::default());
    let rx_connection_handler_ = rx_connection_handler.clone();
    tokio::spawn(async move {
        info!("Spawned RxConnectionHandler thread");
        rx_connection_handler_.handle_incoming_connections(listener2broker_tx, stream_repository_).await;
        warn!("RxConnectionHandler thread going to die");
    });

    tokio::spawn(async move {
        info!("Spawned MetricsServer thread");
        metrics::metrics_server::start_metrics_server(rx_connection_handler, tx_connection_handler, packet_handler).await;
        warn!("MetricsServer thread going to die");
    }
    );


    loop {}
}




