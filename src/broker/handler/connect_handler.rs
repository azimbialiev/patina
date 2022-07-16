use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use log::{debug, info};
use metered::{*};
use tokio::sync::mpsc::Sender;

use crate::{ClientHandler, TopicHandler};
use crate::broker::utils::{generate_client_id, register_clean_session, register_session, send_packet};
use crate::model::control_packet::ControlPacket;
use crate::model::reason_code::ReasonCode;
use crate::session::session_handler::SessionState;

#[derive(Debug)]
pub struct ConnectHandler {
    pub(crate) metrics: ConnectHandlerMetrics,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>
}

#[metered(registry = ConnectHandlerMetrics)]
impl ConnectHandler {

    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub async fn process(&self, socket: &SocketAddr, control_packet: &ControlPacket) -> Result<(), String>{
        let now = Instant::now();
        let mut client_id = generate_client_id();
        if control_packet.has_client_id() {
            debug!("Using client's client_id");
            client_id = control_packet.payload().client_id().to_string();
        }
        info!("CONNECT client: {:?}", client_id);

        if let Some(previous_socket) = self.client_handler.register(&socket, &client_id) {
            info!("Found a previous connection on socket {:?} for client_id {:?}", previous_socket, client_id);
            let disconnect_packet = ControlPacket::disconnect(ReasonCode::SessionTakenOver);
            send_packet(previous_socket, &disconnect_packet, &self.to_listener).await;
        }

        let mut session_present = false;
        if control_packet.variable_header().connect_flags().clean_start_flag() {
            debug!("Creating clean session for client: {:?}", client_id);
            register_clean_session(&client_id);
            self.topic_handler.unsubscribe_all(&client_id);
        } else {
            session_present = match register_session(&client_id) {
                SessionState::SessionPresent => true,
                SessionState::CleanSession => false
            };
        }
        let connack_packet = ControlPacket::connack(session_present);
        send_packet(socket.to_owned(), &connack_packet, &self.to_listener).await;
        //TODO Check Auth
        //TODO Check previous session using client_id
        //TODO Check clean_start
        debug!("Connect handling took {}ms", now.elapsed().as_millis());
        Ok(())
    }


    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Arc<Sender<(Vec<SocketAddr>, ControlPacket)>>) -> Self {
        Self { metrics: ConnectHandlerMetrics::default(), client_handler, topic_handler, to_listener }
    }
}