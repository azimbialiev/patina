use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use metered::{*};
use rand;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::broker::packet_handler::PacketHandler;

use crate::model::control_packet::ControlPacket;


#[derive(Debug)]
pub struct Broker {
    pub(crate) metrics: BrokerMetrics,
    pub(crate) packet_handler: Arc<PacketHandler>,
}

#[metered(registry = BrokerMetrics)]
impl Broker {
    pub async fn handle_packets<'a>(&self,
                                    mut listener2broker: Receiver<(SocketAddr, ControlPacket)>,
                                    broker2listener: Sender<(Vec<SocketAddr>, ControlPacket)>,
    ) {
        info!("Broker::handle_packets");

        loop {
            if let Some((socket, control_packet)) = listener2broker.recv().await {
                let to_listener = broker2listener.clone();
                let handler = self.packet_handler.clone();
                tokio::spawn(async move {
                    match handler.process_message(socket, control_packet, to_listener).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Can't process packet from socket {}. {}", socket, err);
                        }
                    };
                });
            }
        }
    }
    pub fn new(packet_handler: Arc<PacketHandler>) -> Self {
        Self { metrics: BrokerMetrics::default(), packet_handler }
    }
}




