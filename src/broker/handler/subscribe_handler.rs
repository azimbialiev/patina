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
pub struct SubscribeHandler {
    pub(crate) metrics: SubscribeHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>

}

#[metered(registry = SubscribeHandlerMetrics)]
impl SubscribeHandler {

    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub async fn process(&self, socket: &SocketAddr, control_packet: &ControlPacket) -> Result<(), String> {
        let client_id = self.client_handler.get_client_id(&socket)?;
        let topic_filters = control_packet.payload().topic_filters();
        info!("SUBSCRIBE client: {:?} to topics: {:?}", client_id, topic_filters);

        let mut reason_codes = Vec::with_capacity(topic_filters.len());
        for topic_filter in topic_filters {
            self.topic_handler.subscribe(&client_id, topic_filter.topic_filter());
            reason_codes.push(ReasonCode::GrantedQoS0);
            debug!("Subscribed client {:?} to topic {:?}", client_id, topic_filter.topic_filter());
        }
        let suback_packet = ControlPacket::suback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

        send_packet(socket.to_owned(), &suback_packet, &self.to_listener).await;
        Ok(())
    }


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Self {
        Self { metrics: SubscribeHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}