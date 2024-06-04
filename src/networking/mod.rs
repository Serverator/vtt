use crate::prelude::*;

pub mod client;
pub mod protocol;
pub mod shared;
pub mod asset_sharing;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[derive(Default)]
pub struct NetworkingPlugin {
    pub headless: bool,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            shared::SharedPlugin,
            client::ClientPlugin {
                headless: self.headless,
            },
        ));
        
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(server::ServerPlugin);

        app.add_plugins(protocol::ProtocolPlugin);
    }
}
