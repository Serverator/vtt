use crate::prelude::*;

mod client;
mod server;
mod shared;
pub mod protocol;

#[derive(Default)]
pub struct NetworkingPlugin {
	pub headless: bool,
}

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			shared::SharedPlugin,
			client::ClientPlugin,
			server::ServerPlugin,
			protocol::ProtocolPlugin
		));
	}
}