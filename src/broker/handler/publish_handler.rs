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
pub struct PublishHandler {
    pub(crate) metrics: PublishHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>

}

#[metered(registry = PublishHandlerMetrics)]
impl PublishHandler {

    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub async fn process(&self, socket: &SocketAddr, control_packet: &ControlPacket) -> Result<(), String>{
        let client_id = self.client_handler.get_client_id(&socket)?;
        if control_packet.fixed_header().qos_level() == &QoSLevel::AtLeastOnce {
            trace!("Sending PUBACK for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
            let puback_packet = ControlPacket::puback(control_packet.variable_header().packet_identifier_opt());
            send_packet(socket.to_owned(), &puback_packet, &self.to_listener).await;
        } else if control_packet.fixed_header().qos_level() == &QoSLevel::ExactlyOnce {
            trace!("Sending PUBREC for {:?} Packet Identifier to client {:?}", control_packet.variable_header().packet_identifier_opt(), client_id);
            let pubrec_packet = ControlPacket::pubrec(control_packet.variable_header().packet_identifier_opt());
            send_packet(socket.to_owned(), &pubrec_packet, &self.to_listener).await;
        }
        let topic_filter = control_packet.variable_header().topic_name();
        let subscribers =self.topic_handler.find_subscribers(topic_filter);
        info!("PUBLISH client: {:?} to topic:{:?}. Subscribers count: {:?}", client_id, topic_filter, subscribers.len());
        trace!("Found subscribers {:?} for topic {:?}", subscribers, topic_filter);

        persist_packets(&subscribers, &control_packet);
        let clients = subscribers
            .iter()
            .map(|receiver| {
                self.client_handler.get_socket(receiver)
            })
            .filter(Result::is_ok)
            .map(|c| c.unwrap().clone())
            .filter(|receiver| { receiver.ne(&socket) })
            .collect();
        send_packets(clients, control_packet, &self.to_listener).await;
        Ok(())
    }


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Self {
        Self { metrics: PublishHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}