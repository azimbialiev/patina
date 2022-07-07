use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, trace};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{mpsc, Mutex};
use crate::connection::connection::{encode_packet, write_buffer};
use metered::{metered, Throughput, HitCount};

use crate::serdes::mqtt::{ControlPacket, ControlPacketType};

#[derive(Default, Debug)]
#[derive(serde::Serialize)]
pub struct TxConnectionHandler {
    pub(crate) metrics: TxConnectionHandlerMetrics,
}

#[metered(registry = TxConnectionHandlerMetrics)]
impl TxConnectionHandler {
    pub async fn handle_outgoing_connections(&self, mut broker2listener: mpsc::Receiver<(SocketAddr, ControlPacket)>, client2write_half: Arc<Mutex<HashMap<SocketAddr, OwnedWriteHalf>>>) {
        loop {
            if let Some((socket, packet)) = broker2listener.recv().await {
                let mut lock = client2write_half.lock().await;
                let client_tx = lock.get_mut(&socket).expect("panic client2write_half");
                if Self::is_disconnection(&packet).await {
                    debug!("Handling disconnection for client {:?}", socket);
                    let mut lock = client2write_half.lock().await;
                    client_tx.shutdown().await.expect("panic shutdown write half");
                    lock.remove(&socket);
                } else {
                    debug!("Sending packet {:?} to {:?}", packet.fixed_header().packet_type(), socket);
                    self.send_packet(&socket, &packet, client_tx).await
                }
            }
        }
    }

    async fn is_disconnection(packet: &ControlPacket) -> bool {
        if packet.fixed_header().packet_type() == ControlPacketType::DISCONNECT {
            return true;
        }
        return false;
    }

    #[measure([HitCount, Throughput])]
    async fn send_packet(&self, socket: &SocketAddr, packet: &ControlPacket, stream: &mut OwnedWriteHalf) {
        match encode_packet(packet) {
            Ok(buffer) => {
                trace!("Successfully encoded packet");
                match write_buffer(&buffer, stream).await {
                    Ok(_) => {
                        trace!("Successfully sent packet");
                    }
                    Err(err) => {
                        error!("Can't send data to client {:?}: {:?}", socket, err);
                    }
                }
            }
            Err(err) => {
                error!("Can't encode Control Packet: {:?}", err);
                //return Err(format!("Can't encode Control Packet: {:?}", err));
            }
        };
    }
}
