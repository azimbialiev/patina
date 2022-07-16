use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{error, info};
use metered::{*};
use tokio::sync::mpsc::Receiver;

use crate::broker::packet_dispatcher::PacketDispatcher;
use crate::model::control_packet::ControlPacket;

#[derive(Debug)]
pub struct Broker {
    pub(crate) metrics: BrokerMetrics,
    pub(crate) packet_dispatcher: Arc<PacketDispatcher>,
}

#[metered(registry = BrokerMetrics)]
impl Broker {

    //#[tokio::main(flavor = "multi_thread")]
    #[tokio::main(flavor = "multi_thread", worker_threads = 4)]
    //#[tokio::main(flavor = "current_thread")]
    pub async fn handle_packets<'a>(&self,
                                    mut listener2broker: Receiver<(SocketAddr, ControlPacket)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Broker::handle_packets");
        let packet_handler = self.packet_dispatcher.clone();
        loop {
            if let Some((socket, control_packet)) = listener2broker.recv().await {
                let handler = packet_handler.clone();
                tokio::spawn(async move {
                    match handler.process_message(socket, control_packet).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Can't process packet from socket {}. {}", socket, err);
                        }
                    };
                });
            }
        }
    }
    pub fn new(packet_handler: Arc<PacketDispatcher>) -> Self {
        Self { metrics: BrokerMetrics::default(), packet_dispatcher: packet_handler }
    }
}




