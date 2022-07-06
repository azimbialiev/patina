#[macro_use]
extern crate lazy_static;
extern crate core;

use std::collections::HashMap;
use std::sync::Arc;

use log4rs;
use log::{info, warn};
use tokio::sync::{mpsc, Mutex};
use crate::broker::packet_handler::PacketHandler;

use crate::connection::connection_handler::ConnectionHandler;
use crate::topic::topic_handler::TopicHandler;


mod tests;
mod topic;
mod connection;
mod serdes;
mod broker;
mod session;
mod traits;

pub fn init_logging() {
    log4rs::init_file("config/log4rs.yaml", Default::default());
}


#[tokio::main(flavor = "multi_thread",worker_threads = 10)]
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
    tokio::spawn(async move {
        info!("Spawned Broker thread");
        PacketHandler::handle_packets(listener2broker_rx, broker2listener_tx, broker2topic_handler_tx).await;
        warn!("Broker thread going to die");
    });
    let client2write_half_ = client2write_half.clone();

    tokio::spawn(async move {
        info!("Spawned ConnectionHandler incoming connections thread");
        ConnectionHandler::handle_outgoing_connections(broker2listener_rx, client2write_half_).await;
        warn!("ConnectionHandler incoming connections thread going to die");
    });

    let client2write_half_ = client2write_half.clone();
    tokio::spawn(async move {
        info!("Spawned ConnectionHandler outgoing connections thread");
        ConnectionHandler::handle_incoming_connections(1883, listener2broker_tx, client2write_half_).await;
        warn!("ConnectionHandler outgoing connections thread going to die");
    });

    loop {}
}




