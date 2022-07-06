#[macro_use]
extern crate lazy_static;
extern crate core;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log4rs;
use log::{info, warn};
use metrics::{Counter, Gauge, GaugeValue, Histogram, Key, KeyName, Recorder, Unit};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use tokio::sync::{mpsc, Mutex};

use crate::broker::Broker;
use crate::connection_handler::ConnectionHandler;
use crate::topic_handler::TopicHandler;

mod connection_handler;
mod mqtt;
mod decoder;
mod broker;
mod connection;
mod encoder;
mod topic_handler;
mod session;
mod tests;

struct LogRecorder;

impl Recorder for LogRecorder {

    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: &'static str) {
        todo!()
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: &'static str) {
        todo!()
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: &'static str) {
        todo!()
    }

    fn register_counter(&self, key: &Key) -> Counter {
        todo!()
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        todo!()
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        todo!()
    }
}

static RECORDER: LogRecorder = LogRecorder;

lazy_static! {
     static ref broker_instance: Broker = {
        Broker::new()
    };
}

pub fn init_logging() {
    log4rs::init_file("config/log4rs.yaml", Default::default());
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    let builder = PrometheusBuilder::new();
    builder
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .install()
        .expect("failed to install Prometheus recorder");

    metrics::set_recorder(&RECORDER);

    connection::init_connection_metric();


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
        broker_instance.handle_packets(listener2broker_rx, broker2listener_tx, broker2topic_handler_tx).await;
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




