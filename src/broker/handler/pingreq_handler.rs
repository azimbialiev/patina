use std::net::SocketAddr;
use std::sync::Arc;
use log::{debug, info, trace};
use metered::{*};
use tokio::sync::mpsc::Sender;
use crate::broker::utils::{generate_client_id, persist_packets, register_clean_session, register_session, send_packet, send_packets};
use crate::{ClientHandler, TopicHandler};
use crate::model::control_packet::ControlPacket;
use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;
use crate::session::session_handler::SessionState;

#[derive(Debug)]
pub struct PingreqHandler {
    pub(crate) metrics: PingreqHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>

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


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Self {
        Self { metrics: PingreqHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}