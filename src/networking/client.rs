use std::net::{Ipv4Addr, SocketAddrV4};

use crate::prelude::*;
use lightyear::{connection::netcode::PRIVATE_KEY_BYTES, prelude::*};
use client::*;
use rand::RngCore;

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {

		let config = ClientConfig {
			net: NetConfig::Netcode { 
				auth: Authentication::Manual { 
					server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5000)), 
					client_id: rand::thread_rng().next_u64(),
					private_key: [0; PRIVATE_KEY_BYTES],
					protocol_id: 0,
				}, 
				config: NetcodeConfig::default(),
				io: IoConfig::default(),
			},
			..default()
		};

		app.add_plugins(ClientPlugins::new(config));

		app.add_systems(Update, recieve_message);
	}
}

fn recieve_message(
	mut messages: EventReader<MessageEvent<TextMessage>>,
) {
	for message in messages.read() {
		info!("Client recieved message: {}", message.message.0);
	}
}