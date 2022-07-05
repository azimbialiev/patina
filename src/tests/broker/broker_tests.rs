#[cfg(test)]
mod broker_tests {
    use std::net::{IpAddr, SocketAddr};

    use log::{info, log, warn};
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::{Receiver, Sender};

    use crate::{init_logging, TopicHandler};
    use crate::broker::Broker;
    use crate::mqtt::{ControlPacket, ControlPacketType, QoSLevel};
    use crate::tests::broker::broker_tests_data::{create_connect_packet, create_publish_packet_qos1, create_subscribe_packet};

    #[derive(Debug)]
    pub struct Channels {
        listener2broker_tx: Sender<(SocketAddr, ControlPacket)>,
        broker2listener_rx: Receiver<(SocketAddr, ControlPacket)>,
    }

    lazy_static! {

        static ref BROKER_INSTANCE: Broker = {
            Broker::new()

        };

}

    async fn spinup_broker() -> Channels {
        let (listener2broker_tx, listener2broker_rx) = mpsc::channel(32);
        let (broker2listener_tx, broker2listener_rx) = mpsc::channel(32);
        let (broker2topic_handler_tx, broker2topic_handler_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            info!("Spawned TopicHandler thread");
            TopicHandler::handle_topics(broker2topic_handler_rx).await;
            warn!("TopicHandler thread going to die");
        });
        tokio::spawn(async move {
            info!("Spawned broker thread");
            BROKER_INSTANCE.handle_packets(listener2broker_rx, broker2listener_tx, broker2topic_handler_tx).await;
            warn!("Broker thread going to die");
        });
        Channels {
            listener2broker_tx,
            broker2listener_rx,
        }
    }

    fn create_socket(port: u16) -> SocketAddr {
        return SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);
    }


    async fn send_packet_to_broker(tx_socket: &SocketAddr, channels: &mut Channels, packet: &ControlPacket) -> (SocketAddr, ControlPacket) {
        channels.listener2broker_tx.send((tx_socket.clone(), packet.clone())).await.expect("can't send packet to broker");
        return channels.broker2listener_rx.recv().await.expect("can't read packet from broker");
    }

    async fn read_packet_from_broker(channels: &mut Channels) -> (SocketAddr, ControlPacket) {
        return channels.broker2listener_rx.recv().await.expect("can't read packet from broker");
    }

    #[tokio::test]
    async fn simulate_connect() {
        init_logging();
        let tx_socket = create_socket(0001);
        let mut channels = spinup_broker().await;
        let connect_packet = create_connect_packet(String::from("simulate_connect"));
        let (rx_socket, connack_packet) = send_packet_to_broker(&tx_socket, &mut channels, &connect_packet).await;
        assert_eq!(rx_socket, tx_socket);
        assert_eq!(connack_packet.fixed_header().packet_type(), ControlPacketType::CONNACK);
    }

    #[tokio::test]
    async fn simulate_subscribe_unsubscribe() {
        init_logging();
        let tx_socket = create_socket(0001);
        let topic = String::from("test/qos1");
        let mut channels = spinup_broker().await;

        let connect_packet = create_connect_packet(String::from("simulate_subscribe_unsubscribe"));
        send_packet_to_broker(&tx_socket, &mut channels, &connect_packet).await;

        let subscribe_packet = create_subscribe_packet(0, topic.clone(), QoSLevel::AtLeastOnce);
        let (res_tx_socket, suback_packet) = send_packet_to_broker(&tx_socket, &mut channels, &subscribe_packet).await;
        assert_eq!(res_tx_socket, tx_socket);
        assert_eq!(suback_packet.fixed_header().packet_type(), ControlPacketType::SUBACK);
    }

    #[tokio::test]
    async fn simulate_publish_qos1() {
        init_logging();
        let tx_socket = create_socket(0001);
        let rx_socket = create_socket(0002);
        let topic = String::from("test/qos1");
        let mut channels = spinup_broker().await;

        let connect_packet = create_connect_packet(String::from("simulate_publish_qos1_tx"));
        send_packet_to_broker(&tx_socket, &mut channels, &connect_packet).await;

        let connect_packet = create_connect_packet(String::from("simulate_publish_qos1_rx"));
        send_packet_to_broker(&rx_socket, &mut channels, &connect_packet).await;

        let subscribe_packet = create_subscribe_packet(0, topic.clone(), QoSLevel::AtLeastOnce);
        send_packet_to_broker(&rx_socket, &mut channels, &subscribe_packet).await;

        let publish_packet = create_publish_packet_qos1(0, topic);
        let (res_tx_socket, puback_packet) = send_packet_to_broker(&tx_socket, &mut channels, &publish_packet).await;

        assert_eq!(res_tx_socket, tx_socket);
        assert_eq!(puback_packet.fixed_header().packet_type(), ControlPacketType::PUBACK);
        assert_eq!(puback_packet.variable_header().packet_identifier(), publish_packet.variable_header().packet_identifier());

        let (res_rx_socket, publish_packet) = read_packet_from_broker(&mut channels).await;
        assert_eq!(res_rx_socket, rx_socket);
        assert_eq!(publish_packet.fixed_header().qos_level(), &QoSLevel::AtLeastOnce);
        assert_eq!(publish_packet.fixed_header().packet_type(), ControlPacketType::PUBLISH);
    }
}