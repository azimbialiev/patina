use std::fmt::Debug;
use std::net::SocketAddr;

use log::{debug, error, info, trace, warn};
use metered::{*};
use rand;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::broker::packet_handler::PacketHandler;

use crate::model::control_packet::ControlPacket;
use crate::topic::topic_handler::TopicCommand;


#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct Broker {
    pub(crate) metrics: BrokerMetrics,
    pub(crate) packet_handler: PacketHandler,
}

#[metered(registry = BrokerMetrics)]
impl Broker {
    pub async fn handle_packets<'a>(&self, mut listener2broker: Receiver<(SocketAddr, ControlPacket)>, broker2listener: Sender<(SocketAddr, ControlPacket)>, broker2topic_handler: Sender<TopicCommand>) {
        info!("Broker::handle_packets");

        loop {
            match listener2broker.recv().await {
                None => {
                    warn!("Channel has been closed!");
                }
                Some((client, control_packet)) => {
                    let to_listener = broker2listener.clone();
                    let to_topic_handler = broker2topic_handler.clone();
                    let handler = self.packet_handler.clone();
                    tokio::spawn(async move {
                        handler.process_message(client, control_packet, to_listener, to_topic_handler).await.expect("panic process_message");
                    });
                }
            }
        }
    }
}




