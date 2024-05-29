use crate::prelude::*;

mod chat;
mod connection;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((chat::ChatWindowPlugin, connection::ConnectionWindowPlugin));
    }
}
