use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, info};
use metered::{*};
use tokio::sync::mpsc::Sender;

use crate::{ClientHandler, TopicHandler};
use crate::broker::utils::send_packet;
use crate::model::control_packet::ControlPacket;
use crate::model::reason_code::ReasonCode;

#[derive(Debug)]
pub struct UnsubscribeHandler {
    pub(crate) metrics: UnsubscribeHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>

}

#[metered(registry = UnsubscribeHandlerMetrics)]
impl UnsubscribeHandler {

    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub async fn process(&self, socket: &SocketAddr, control_packet: &ControlPacket) -> Result<(), String> {
        let client_id = self.client_handler.get_client_id(&socket)?;
        let topic_filters = control_packet.payload().topic_filters();
        info!("UNSUBSCRIBE client: {:?} from topics: {:?}", client_id, topic_filters);
        let mut reason_codes = Vec::with_capacity(topic_filters.len());
        for topic_filter in topic_filters {
            self.topic_handler.unsubscribe(&client_id, topic_filter.topic_filter());
            reason_codes.push(ReasonCode::Success);
            debug!("Unsubscribed client {:?} from topic {:?}", client_id, topic_filter.topic_filter());
        }
        let unsuback_packet = ControlPacket::unsuback(control_packet.variable_header().packet_identifier_opt(), reason_codes);

        send_packet(socket.to_owned(), &unsuback_packet, &self.to_listener).await;
        Ok(())
    }


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>) -> Self {
        Self { metrics: UnsubscribeHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}