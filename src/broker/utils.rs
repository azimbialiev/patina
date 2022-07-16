use std::net::SocketAddr;

use chrono::Local;
use dashmap::DashMap;
use log::{error, trace};
use rand::Rng;
use tokio::sync::mpsc::Sender;

use crate::model::control_packet::ControlPacket;
use crate::session::session_handler::{SessionHandler, SessionState};

lazy_static! {

    static  ref id2session: DashMap<String, SessionHandler> = {
        let map = DashMap::new();
        map
    };
}

pub fn persist_packets(client_ids: &Vec<String>, publish_packet: &ControlPacket) {
    trace!("Broker::persist_packets");
    for client_id in client_ids {
        id2session.get_mut(client_id).unwrap()
            .register_publish(client_id.clone(), publish_packet);
    }
}

pub fn register_session(client_id: &String) -> SessionState {
    trace!("Broker::register_session");
    if id2session.contains_key(client_id) {
        return SessionState::SessionPresent;
    }

    return match id2session.insert(client_id.clone(), SessionHandler::new()) {
        None => {
            trace!("Created new Session for client: {:?}", client_id);
            SessionState::CleanSession
        }
        Some(session) => {
            error!("Need to handle 'session taken over' case");
            SessionState::SessionPresent
        }
    };
}

pub fn register_clean_session(client_id: &String) {
    trace!("Broker::register_clean_session");
    id2session.insert(client_id.clone(), SessionHandler::new());
}

pub async fn send_packet(socket: SocketAddr, packet: &ControlPacket, to_listener: &Sender<(Vec<SocketAddr>, ControlPacket)>) {
    return send_packets(vec![socket], packet, to_listener).await;
}

pub async fn send_packets(sockets: Vec<SocketAddr>, control_packet: &ControlPacket, to_listener: &Sender<(Vec<SocketAddr>, ControlPacket)>) {
    trace!("Broker::send_packets");
    match to_listener.send((sockets, control_packet.clone())).await {
        Ok(_) => {
            trace!("Successfully sent packet");
        }
        Err(err) => {
            error!("Can't send data to sockets: {:?}",  err);
        }
    }

    trace!("Done sending packets");
}


pub fn generate_client_id() -> String {
    trace!("Broker::generate_client_id");
    let random_prefix: u64 = rand::thread_rng().gen();
    let now = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    return random_prefix.to_string() + &now.to_string();
}
