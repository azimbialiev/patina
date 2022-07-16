use std::net::SocketAddr;
use std::sync::Arc;

use log::debug;
use metered::{*};
use tokio::sync::mpsc::Sender;

use crate::{ClientHandler, TopicHandler};
use crate::broker::utils::send_packet;
use crate::model::control_packet::ControlPacket;

#[derive(Debug)]
pub struct PingreqHandler {
    pub(crate) metrics: PingreqHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>

}

#[metered(registry = PingreqHandlerMetrics)]
impl PingreqHandler {

    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub async fn process(&self, socket: &SocketAddr, control_packet: &ControlPacket) -> Result<(), String> {
        let client_id = self.client_handler.get_client_id(&socket)?;
        debug!("PINGREQ from client {:?}", client_id);
        let pingresp_packet = ControlPacket::pingresp();
        send_packet(socket.to_owned(), &pingresp_packet, &self.to_listener).await;
        Ok(())
    }


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>) -> Self {
        Self { metrics: PingreqHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}