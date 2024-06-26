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
            transport: ServerTransport::WebTransportServer {
                server_addr: SERVER_ADDR,
                certificate: Identity::self_signed(["localhost"]).unwrap(),
            },
            compression: CompressionConfig::Zstd { level: 3 },
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
            .add_systems(Startup, replicate_resources)
            .add_systems(
                Update,
                (recieve_message, replicate_cursors, despawn_cursors)
                    .run_if(in_state(NetworkingState::Started)),
            );
    }
}

fn replicate_resources(mut commands: Commands) {
    commands.replicate_resource::<PlayerData, SequencedReliable>(NetworkTarget::All)
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
            .send_message_to_target::<UnorderedReliable, _>(&chat_message, NetworkTarget::All)
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
            .send_message_to_target::<UnorderedReliable, _>(&chat_message, NetworkTarget::All)
            .unwrap();
    }

    for disconnected in disconnected.read() {
        info!("Player disconnected: {}", disconnected.client_id.to_bits());
        let chat_message = ChatMessage::Disconnected(disconnected.client_id.to_bits());
        connection
            .send_message_to_target::<UnorderedReliable, _>(&chat_message, NetworkTarget::All)
            .unwrap();

        clients.remove(&disconnected.client_id.to_bits());
    }
}

fn replicate_cursors(
    mut commands: Commands,
    client_id: Option<Res<super::client::ClientId>>,
    cursors: Query<(Entity, &Replicated), (With<Cursor>, Added<Replicated>)>,
) {
    for (entity, replicated) in cursors.iter() {
        let mut entity = commands.entity(entity);
        let client_id = replicated.from.unwrap_or(ClientId::Local(client_id.as_ref().unwrap().0));

        entity.insert((
            server::Replicate {
                target: ReplicationTarget {
                    target: NetworkTarget::AllExceptSingle(client_id),
                },
                ..default()
            },
            Owner(client_id.to_bits()),
        ));
    }
}

fn despawn_cursors(
    mut commands: Commands,
    mut disconnected: EventReader<DisconnectEvent>,
    cursors: Query<(Entity, &Replicated), With<Cursor>>,
) {
    for disconnect in disconnected.read() {
        let client_id = disconnect.client_id;

        info!("Despawning entity");

        if let Some((cursor, _)) = cursors.iter().find(|x| x.1.client_id() == client_id) {
            commands.entity(cursor).despawn();
        }
    }
}
