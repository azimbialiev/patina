#[macro_use]
extern crate lazy_static;
extern crate core;

use std::collections::HashMap;
use std::sync::Arc;

use log4rs;
use log::{info, warn};
use nameof::name_of_type;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex, RwLock};
use crate::broker::packet_handler::PacketHandler;
use crate::connection::rx_connection_handler::{RxConnectionHandler};
use crate::connection::tx_connection_handler::{TxConnectionHandler};
use crate::metrics::metrics_registry::ServiceMetricRegistry;
use crate::topic::topic_handler::TopicHandler;

mod tests;
mod topic;
mod connection;
mod serdes;
mod broker;
mod session;
mod traits;
mod metrics;

pub fn init_logging() {
    log4rs::init_file("config/log4rs.yaml", Default::default());
}


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    info!("MQTT SERVER");
    let (listener2broker_tx, listener2broker_rx) = mpsc::channel(1000);
    let (broker2listener_tx, broker2listener_rx) = mpsc::channel(1000);
    let client2write_half = Arc::new(Mutex::new(HashMap::new()));
    let (broker2topic_handler_tx, broker2topic_handler_rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        info!("Spawned TopicHandler thread");
        TopicHandler::handle_topics(broker2topic_handler_rx).await;
        warn!("TopicHandler thread going to die");
    });
    let packet_handler = Arc::new(RwLock::new(PacketHandler::default()));
    let packet_handler_ = packet_handler.clone();
    tokio::spawn(async move {
        info!("Spawned Broker thread");
        let lock = packet_handler_.read().await;
        lock.handle_packets(listener2broker_rx, broker2listener_tx, broker2topic_handler_tx).await;
        warn!("Broker thread going to die");
    });

    let client2write_half_ = client2write_half.clone();
    let tx_connection_handler = Arc::new(RwLock::new(TxConnectionHandler::default()));
    let tx_connection_handler_ = tx_connection_handler.clone();
    tokio::spawn(async move {
        info!("Spawned TxConnectionHandler thread");
        let lock = tx_connection_handler_.read().await;
        lock.handle_outgoing_connections(broker2listener_rx, client2write_half_).await;
        warn!("TxConnectionHandler incoming connections thread going to die");
    });

    let client2write_half_ = client2write_half.clone();
    let rx_connection_handler = Arc::new(RwLock::new(RxConnectionHandler::default()));
    let rx_connection_handler_ = rx_connection_handler.clone();
    tokio::spawn(async move {
        info!("Spawned RxConnectionHandler thread");
        let lock = rx_connection_handler_.read().await;
        lock.handle_incoming_connections(listener2broker_tx, client2write_half_).await;
        warn!("RxConnectionHandler outgoing connections thread going to die");
    });

    tokio::spawn({
        async move {
            println!("Prometheus metrics exposed on 127.0.0.1:9000");
            let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await.unwrap();

            loop {
                let registry = &ServiceMetricRegistry {
                    rx_connection_handler: &rx_connection_handler.read().await.metrics,
                    tx_connection_handler: &tx_connection_handler.read().await.metrics,
                    packet_handler: &packet_handler.read().await.metrics,
                };
                let (mut stream, _) = listener.accept().await.unwrap();

                let mut globals = HashMap::new();
                globals.insert(name_of_type!(RxConnectionHandler), "serde_prometheus_example");
                globals.insert(name_of_type!(TxConnectionHandler), "serde_prometheus_example");

                let serialized = serde_prometheus::to_string(
                    registry,
                    Some("patina"),
                    globals,
                )
                    .unwrap();

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    serialized.len(),
                    serialized
                );
                stream.write(response.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            }
        }
    });

    loop {}
}




