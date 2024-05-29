use std::net::{Ipv4Addr, SocketAddrV4};

use crate::prelude::*;
use client::*;
use lightyear::{connection::netcode::PRIVATE_KEY_BYTES, prelude::*};
use rand::RngCore;

use super::shared::DEFAULT_PORT;

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let config = ClientConfig {
            net: NetConfig::Netcode {
                auth: Authentication::Manual {
                    server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::new(127, 0, 0, 1),
                        DEFAULT_PORT,
                    )),
                    client_id: rand::thread_rng().next_u64(),
                    private_key: [0; PRIVATE_KEY_BYTES],
                    protocol_id: 0,
                },
                config: NetcodeConfig::default(),
                io: IoConfig {
                    transport: ClientTransport::UdpSocket(std::net::SocketAddr::V4(
                        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0),
                    )),
                    ..default()
                },
            },

            ..default()
        };

        app.add_systems(OnEnter(NetworkingState::Connected), send_player_info);

        //app.add_systems(Startup, connect);

        app.add_plugins(ClientPlugins::new(config));
        app.insert_resource(Player {
            name: String::from("Player"),
            color: [255; 3],
        });

        app.add_systems(Update, recieve_message);
    }
}

fn send_player_info(mut connection: ResMut<ConnectionManager>, player: Res<Player>) {
    _ = connection.send_message::<UnorderedReliableChannel, Player>(&player);
}

fn recieve_message(mut messages: EventReader<MessageEvent<ChatMessage>>) {
    for message in messages.read() {
        match &message.message {
            ChatMessage::Message(client, message) => info!("{client}: {message}"),
            ChatMessage::Connected(client) => info!("Client {client} connected"),
            ChatMessage::Disconnected(client) => info!("Client {client} disconnected"),
        }
    }
}
