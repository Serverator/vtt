use std::net::{Ipv4Addr, SocketAddrV4};

use crate::prelude::*;
use lightyear::prelude::{server::*, *};

use super::shared::DEFAULT_PORT;
const SERVER_ADDR: std::net::SocketAddr =
    std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), DEFAULT_PORT));

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let io = IoConfig {
            transport: ServerTransport::UdpSocket(SERVER_ADDR),
            ..default()
        };

        let net = NetConfig::Netcode {
            io,
            config: NetcodeConfig::default(),
        };

        let config = ServerConfig {
            net: vec![net],
            ..default()
        };

        app.add_plugins(ServerPlugins::new(config))
            .init_resource::<PlayerData>()
            .init_resource::<ConnectedClients>()
            .add_systems(
                Update,
                recieve_message.run_if(in_state(NetworkingState::Started)),
            );
    }
}

fn recieve_message(
    mut messages: EventReader<MessageEvent<SendMessage>>,
    mut player_updated: EventReader<MessageEvent<Player>>,
    mut connected: EventReader<ConnectEvent>,
    mut disconnected: EventReader<DisconnectEvent>,
    mut player_list: ResMut<PlayerData>,
    mut clients: ResMut<ConnectedClients>,
    mut connection: ResMut<ConnectionManager>,
) {
    for player_updated in player_updated.read() {
        player_list.0.insert(
            player_updated.context.to_bits(),
            player_updated.message.clone(),
        );
    }

    for connected in connected.read() {
        info!("Player connected: {}", connected.client_id.to_bits());
        let chat_message = ChatMessage::Connected(connected.client_id.to_bits());

        clients.insert(connected.client_id.to_bits());

        connection
            .send_message_to_target::<UnorderedReliableChannel, _>(
                &chat_message,
                NetworkTarget::All,
            )
            .unwrap();
    }

    for message in messages.read() {
        info!(
            "Server recieved message from {}: {}",
            message.context.to_bits(),
            message.message.0
        );
        let chat_message =
            ChatMessage::Message(message.context.to_bits(), message.message.0.clone());
        connection
            .send_message_to_target::<UnorderedReliableChannel, _>(
                &chat_message,
                NetworkTarget::All,
            )
            .unwrap();
    }

    for disconnected in disconnected.read() {
        info!("Player disconnected: {}", disconnected.client_id.to_bits());
        let chat_message = ChatMessage::Disconnected(disconnected.client_id.to_bits());
        connection
            .send_message_to_target::<UnorderedReliableChannel, _>(
                &chat_message,
                NetworkTarget::All,
            )
            .unwrap();

        clients.remove(&disconnected.client_id.to_bits());
    }
}
