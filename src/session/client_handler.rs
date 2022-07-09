
use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;

use log::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct ClientHandler {
    socket2id: Arc<DashMap<SocketAddr, String>>,
    id2socket: Arc<DashMap<String, SocketAddr>>,
}

impl Default for ClientHandler{
    fn default() -> Self {
        Self { socket2id: Arc::new(DashMap::new()), id2socket: Arc::new(DashMap::new()) }
    }
}

impl ClientHandler {

    pub fn get_client_id(&self, socket: &SocketAddr) -> Result<String, String> {
        match self.socket2id.get(&socket) {
            None => {
                Err(format!("Can't get any client_id for socket {}", socket))
            }
            Some(client_id) => {
                Ok(client_id.value().clone())
            }
        }
    }

    pub fn get_socket(&self, client_id: &String) -> Result<SocketAddr, String> {
        match self.id2socket.get(client_id) {
            None => {
                Err(format!("Can't get any socket for client_id {}", client_id))
            }
            Some(socket) => {
                Ok(socket.value().clone())
            }
        }
    }

    pub fn register(&self, socket: &SocketAddr, client_id: &String) -> Option<SocketAddr> {
        if self.socket2id.contains_key(&socket) {
            warn!("The socket {} is already registered with client_id {}. New client_id: {}",client_id, self.socket2id.get(&socket).unwrap().to_string(), socket);
        }
        match self.socket2id.insert(socket.clone(), client_id.clone()) {
            None => {
                trace!("Registered socket2id: {:?} -> {:?}", socket, client_id);
            }
            Some(_) => {
                error!("Need to handle 'session taken over' case");
            }
        };
        //let mut id2socket = id2socket.write().await;
        let previous_socket = match self.id2socket.insert(client_id.clone(), socket.clone()) {
            None => {
                trace!("Registered id2socket: {:?} -> {:?}", client_id, socket);
                None
            }
            Some(previous_socket) => {
                info!("Found a previous socket {:?} associated to client {:?}", previous_socket, client_id);
                Some(previous_socket)
            }
        };
        previous_socket
    }

    pub fn unregister(&self, socket: &SocketAddr, client_id: &String) {
        match self.socket2id.remove(&socket) {
            None => {
                trace!("Unregister socket2id: {:?} -> {:?}", socket, client_id);
            }
            Some(_) => {
                error!("Need to handle 'session taken over' case");
            }
        };

        match self.id2socket.remove(client_id) {
            None => {
                trace!("Unregister id2socket: {:?} -> {:?}", client_id, socket);
            }
            Some(previous_socket) => {
                error!("Need to handle 'session taken over' case");
            }
        };
    }
}