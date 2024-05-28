use std::net::{Ipv4Addr, SocketAddrV4};

use crate::prelude::*;
use lightyear::prelude::{*, server::*};

const SERVER_ADDR: std::net::SocketAddr = std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 5000));

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
	fn build(&self, app: &mut App) {
		let io = IoConfig {
			transport: ServerTransport::UdpSocket(SERVER_ADDR),
			..default()
		};

		let net = NetConfig::Netcode {
			io,
			config: NetcodeConfig::default()
		};

		let config = ServerConfig {
			net: vec![ net ],
			..default()
		};

		app .add_plugins(ServerPlugins::new(config))
			.add_systems(Update, recieve_message.run_if(in_state(NetworkingState::Started)));
	}
}

fn recieve_message(
	mut messages: EventReader<MessageEvent<TextMessage>>,
	mut connection: ResMut<ConnectionManager>,
) {
	for message in messages.read() {
		info!("Server recieved message: {}", message.message.0);
		connection.send_message_to_target::<UnorderedReliableChannel, _>(&message.message, NetworkTarget::AllExceptSingle(message.context)).unwrap();
	}
}