mod listener;
mod mqtt;
mod decoder;
mod broker;
mod connection;
mod encoder;
mod tests;
mod topic_handler;
mod session;

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use log4rs;
use log::{info, debug, trace, warn, error};
use metrics::{GaugeValue, Key, Recorder, Unit, register_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::sync::mpsc;
use metrics_util::MetricKindMask;
use tokio::sync::broadcast;
use crate::broker::Broker;

#[macro_use]
extern crate lazy_static;

struct LogRecorder;

impl Recorder for LogRecorder {
    fn register_counter(&self, key: &Key, _unit: Option<Unit>, _description: Option<&'static str>) {}

    fn register_gauge(&self, key: &Key, _unit: Option<Unit>, _description: Option<&'static str>) {}

    fn register_histogram(&self, key: &Key, _unit: Option<Unit>, _description: Option<&'static str>) {}

    fn increment_counter(&self, key: &Key, value: u64) {
        info!("Metrics::counter '{}' -> {}", key, value);
    }

    fn update_gauge(&self, key: &Key, value: GaugeValue) {
        info!("Metrics::gauge '{}' -> {:?}", key, value);
    }

    fn record_histogram(&self, key: &Key, value: f64) {
        info!("Metrics::histogram '{}' -> {}", key, value);
    }
}

static RECORDER: LogRecorder = LogRecorder;

lazy_static! {
     static ref broker_instance: Broker = {
        Broker::new()
    };
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

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
    let (listener2broker_packet_tx, listener2broker_packet_rx) = mpsc::channel(32);
    let (listener2broker_client_tx_tx, listener2broker_client_tx_rx) = mpsc::channel(32);
    let mut listener = listener::Listener::new(1883, listener2broker_packet_tx, listener2broker_client_tx_tx).await;
    tokio::spawn(async move {
        broker_instance.start(listener2broker_packet_rx, listener2broker_client_tx_rx).await;
    });

    loop {
        listener.process().await;
    }
}


