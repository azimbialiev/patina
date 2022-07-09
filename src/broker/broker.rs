use std::fmt::Debug;
use std::net::SocketAddr;

use log::{debug, error, info, trace, warn};
use metered::{*};
use rand;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::broker::packet_handler::PacketHandler;

use crate::model::control_packet::ControlPacket;


#[derive(Default, Debug)]
pub struct Broker {
    pub(crate) metrics: BrokerMetrics,
    pub(crate) packet_handler: PacketHandler,
}

#[metered(registry = BrokerMetrics)]
impl Broker {
    pub async fn handle_packets<'a>(&self,
                                    mut listener2broker: Receiver<(SocketAddr, ControlPacket)>,
                                    broker2listener: Sender<(Vec<SocketAddr>, ControlPacket)>,
    ) {
        info!("Broker::handle_packets");

        loop {
            match listener2broker.recv().await {
                None => {
                    warn!("Channel has been closed!");
                }
                Some((socket, control_packet)) => {
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
    }
}




