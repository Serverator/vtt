use crate::prelude::*;

pub mod client;
pub mod protocol;
pub mod server;
pub mod shared;

#[derive(Default)]
pub struct NetworkingPlugin {
    pub headless: bool,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            shared::SharedPlugin,
            client::ClientPlugin { headless: self.headless },
            server::ServerPlugin,
            protocol::ProtocolPlugin,
        ));
    }
}
