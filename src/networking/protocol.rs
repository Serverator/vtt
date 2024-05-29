use crate::prelude::*;
use bevy::utils::{HashMap, HashSet};
use lightyear::prelude::*;

pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SendMessage>(ChannelDirection::ClientToServer);
        app.add_message::<ChatMessage>(ChannelDirection::ServerToClient);
        app.add_message::<Player>(ChannelDirection::ClientToServer);
        app.register_resource::<PlayerData>(ChannelDirection::ServerToClient);
        app.register_resource::<ConnectedClients>(ChannelDirection::ServerToClient);
        app.add_channel::<UnorderedReliableChannel>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}

#[derive(Channel)]
pub struct UnorderedReliableChannel;

#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct PlayerData(pub HashMap<u64, Player>);

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct ConnectedClients(pub HashSet<u64>);

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub color: [u8; 3],
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: String::from("Player"),
            color: [255; 3],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SendMessage(pub String);

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum ChatMessage {
    Message(u64, String),
    Connected(u64),
    Disconnected(u64),
}
