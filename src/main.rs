extern crate core;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use dashmap::DashMap;

use log::{info, warn};
use log4rs;

use crate::broker::broker::Broker;
use crate::broker::packet_dispatcher::PacketDispatcher;
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


//#[tokio::main(flavor = "multi_thread", worker_threads = 32)]
fn main() {
    init_logging();

    info!("MQTT SERVER");
    let (listener2broker_tx, listener2broker_rx) = tokio::sync::mpsc::channel(1000000);
    let (broker2listener_tx, broker2listener_rx) = tokio::sync::mpsc::channel(1000000);
    let stream_repository = Arc::new(DashMap::new());
    let topic_handler = Arc::new(TopicHandler::default());
    let client_handler = Arc::new(ClientHandler::default());
    let packet_handler = Arc::new(PacketDispatcher::new(client_handler.clone(), topic_handler.clone(), broker2listener_tx));
    let broker = Arc::new(Broker::new(packet_handler.clone()));
    let packet_handler_ = broker.clone();

    let broker_handle = thread::spawn(move || {
        info!("Spawned Broker thread");
        packet_handler_.handle_packets(
            listener2broker_rx,
        );
    });

    // tokio::spawn(async move {
    //     warn!("Broker thread going to die");
    //
    // });

    let stream_repository_ = stream_repository.clone();
    let tx_connection_handler = Arc::new(TxConnectionHandler::new(client_handler.clone(), topic_handler.clone()));
    let tx_connection_handler_ = tx_connection_handler.clone();
    let listener2broker_tx_ = listener2broker_tx.clone();

    let tx_connections_handle = thread::spawn(move || {
        info!("Spawned TxConnectionHandler thread");
        tx_connection_handler_.handle_outgoing_connections(broker2listener_rx, listener2broker_tx_, stream_repository_);
    });

    // tokio::spawn(async move {
    //
    //     warn!("TxConnectionHandler thread going to die");
    // });

    let stream_repository_ = stream_repository.clone();
    let rx_connection_handler = Arc::new(RxConnectionHandler::new());
    let rx_connection_handler_ = rx_connection_handler.clone();

    let rx_connection_handle = thread::spawn(move || {
        info!("Spawned RxConnectionHandler thread");
        rx_connection_handler_.handle_incoming_connections(listener2broker_tx, stream_repository_);
    });

    // tokio::spawn(async move {
    //
    //     warn!("RxConnectionHandler thread going to die");
    // });
    let metrics_handle = thread::spawn(move || {
        info!("Spawned MetricsServer thread");
        metrics::metrics_server::start_metrics_server(rx_connection_handler, tx_connection_handler, broker);
    });


    broker_handle.join().expect("");
    tx_connections_handle.join().expect("");
    rx_connection_handle.join().expect("");
    metrics_handle.join().expect("");


    // tokio::spawn(async move {
    //
    //     warn!("MetricsServer thread going to die");
    // }
    // );
}




