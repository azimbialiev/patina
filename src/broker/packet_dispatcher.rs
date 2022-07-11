use std::net::SocketAddr;
use std::sync::Arc;
use chrono::Local;
use serde::Serializer;
use log::{debug, error, info, trace, warn};
use metered::{*};
use rand::Rng;
use crate::model::control_packet::ControlPacket;
use crate::model::fixed_header::ControlPacketType;
use crate::model::qos_level::QoSLevel;
use crate::model::reason_code::ReasonCode;
use crate::session::session_handler::{SessionHandler, SessionState};
use dashmap::DashMap;
use tokio::sync::mpsc::Sender;
use crate::{ClientHandler, TopicHandler};
use crate::broker::handler::connect_handler::ConnectHandler;
use crate::broker::handler::disconnect_handler::DisconnectHandler;
use crate::broker::handler::pingreq_handler::PingreqHandler;
use crate::broker::handler::publish_handler::PublishHandler;
use crate::broker::handler::pubrec_handler::PubrecHandler;
use crate::broker::handler::pubrel_handler::PubrelHandler;
use crate::broker::handler::subscribe_handler::SubscribeHandler;
use crate::broker::handler::unsubscribe_handler::UnsubscribeHandler;


#[derive(Debug)]
pub struct PacketDispatcher {
    pub(crate) metrics: PacketDispatcherMetrics,
    to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>,
    pub(crate) client_handler: Arc<ClientHandler>,
    pub(crate) topic_handler: Arc<TopicHandler>,
    pub(crate) connect_handler: Arc<ConnectHandler>,
    pub(crate) disconnect_handler: Arc<DisconnectHandler>,
    pub(crate) pingreq_handler: Arc<PingreqHandler>,
    pub(crate) publish_handler: Arc<PublishHandler>,
    pub(crate) pubrec_handler: Arc<PubrecHandler>,
    pub(crate) pubrel_handler: Arc<PubrelHandler>,
    pub(crate) subscribe_handler: Arc<SubscribeHandler>,
    pub(crate) unsubscribe_handler: Arc<UnsubscribeHandler>,
}

#[metered(registry = PacketDispatcherMetrics)]
impl PacketDispatcher {
    #[measure([HitCount, Throughput, InFlight, ResponseTime, ErrorCount])]
    pub(crate) async fn process_message(&self,
                                        socket: SocketAddr,
                                        control_packet: ControlPacket,
    ) -> Result<(), String> {
        debug!("Going to handle control packet: {:?} from client {:?} on socket {:?}",
        control_packet.fixed_header().packet_type(),match self.client_handler.get_client_id(&socket)
            {Err(_) => {String::from("<CLIENT_ID NOT REGISTERED>")}, Ok(client_id) => {client_id.clone()}},
            socket);

        let result = match control_packet.fixed_header().packet_type() {
            ControlPacketType::RESERVED => {}
            ControlPacketType::CONNECT => {
                self.connect_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::CONNACK => {}
            ControlPacketType::PUBLISH => {
                self.publish_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::PUBACK => {}
            ControlPacketType::PUBREC => {
                self.pubrec_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::PUBREL => {
                self.pubrel_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::PUBCOMP => {}
            ControlPacketType::SUBSCRIBE => {
                self.subscribe_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::SUBACK => {}
            ControlPacketType::UNSUBSCRIBE => {
                self.unsubscribe_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::UNSUBACK => {}
            ControlPacketType::PINGREQ => {
                self.pingreq_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::PINGRESP => {}
            ControlPacketType::DISCONNECT => {
                self.disconnect_handler.process(&socket, &control_packet).await?;
            }
            ControlPacketType::AUTH => {}
        };
        Ok(())
    }
    pub fn new(client_handler: Arc<ClientHandler>, topic_handler: Arc<TopicHandler>, to_listener: Sender<(Vec<SocketAddr>, ControlPacket)>) -> Self {
        Self {
            metrics: PacketDispatcherMetrics::default(),
            to_listener: to_listener.clone(),
            client_handler: client_handler.clone(),
            topic_handler: topic_handler.clone(),
            connect_handler: Arc::new(ConnectHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            disconnect_handler: Arc::new(DisconnectHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            pingreq_handler: Arc::new(PingreqHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            publish_handler: Arc::new(PublishHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            pubrec_handler: Arc::new(PubrecHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            pubrel_handler: Arc::new(PubrelHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            subscribe_handler: Arc::new(SubscribeHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
            unsubscribe_handler: Arc::new(UnsubscribeHandler::new(client_handler.clone(), topic_handler.clone(), to_listener.clone())),
        }
    }
}


