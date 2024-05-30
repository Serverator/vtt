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

        app.register_component::<Cursor>(ChannelDirection::Bidirectional)
            .add_interpolation(client::ComponentSyncMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<Owner>(ChannelDirection::ServerToClient)
            .add_interpolation(client::ComponentSyncMode::Once);

        app.register_type::<Cursor>();
        app.register_type::<Replicated>();

        app.add_channel::<UnorderedReliable>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        });
        app.add_channel::<SequencedUnreliable>(ChannelSettings {
            mode: ChannelMode::SequencedUnreliable,
            ..default()
        });
    }
}

#[derive(Channel)]
pub struct UnorderedReliable;

#[derive(Channel)]
pub struct SequencedUnreliable;

#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct PlayerData(pub HashMap<u64, Player>);

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct ConnectedClients(pub HashSet<u64>);

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub color: [u8; 3],
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq, Deref, DerefMut)]
pub struct Owner(pub u64);

#[derive(Component, Clone, Copy, Reflect, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Cursor {
    pub position: Vec2,
}

impl Linear for Cursor {
    fn lerp(start: &Self, other: &Self, t: f32) -> Self {
        Cursor { position: (1.0 - t) * start.position + t * other.position }
    }
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
