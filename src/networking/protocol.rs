use crate::prelude::*;
use bevy::{
    ecs::entity::MapEntities, utils::{HashMap, HashSet}
};
use lightyear::prelude::*;

pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SendMessage>(ChannelDirection::ClientToServer);
        app.add_message::<ChatMessage>(ChannelDirection::ServerToClient);
        app.add_message::<DeselectMessage>(ChannelDirection::ServerToClient);
        app.add_message::<Player>(ChannelDirection::ClientToServer);

        app.register_resource::<PlayerData>(ChannelDirection::ServerToClient);
        app.register_resource::<ConnectedClients>(ChannelDirection::ServerToClient);

        app.register_component::<Cursor>(ChannelDirection::Bidirectional);
        app.register_component::<Owner>(ChannelDirection::ServerToClient);
        app.register_component::<Token>(ChannelDirection::ServerToClient);

        app.register_type::<Token>();
        app.register_type::<Cursor>();
        app.register_type::<Replicated>();
        app.register_type::<Owner>();
        app.register_type::<DeselectMessage>()
            .add_map_entities::<DeselectMessage>();

        app.add_shared_asset::<Image>();

        app.add_channel::<UnorderedReliable>(ChannelSettings {
            mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
            ..default()
        });

        app.add_channel::<SequencedUnreliable>(ChannelSettings {
            mode: ChannelMode::SequencedUnreliable,
            ..default()
        });

        app.add_channel::<SequencedReliable>(ChannelSettings {
            mode: ChannelMode::SequencedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}

#[derive(Channel)]
pub struct UnorderedReliable;

#[derive(Channel)]
pub struct SequencedUnreliable;

#[derive(Channel)]
pub struct SequencedReliable;

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub position: Vec2,
    pub layer: f32,
}

#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct SharedAsset<T> {
    #[deref]
    pub id: Uuid,
    #[reflect(ignore)]
    _spooky: PhantomData<T>,
}

impl<T> SharedAsset<T> {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            id: uuid,
            _spooky: PhantomData,
        }
    }
}

impl<T> PartialEq for SharedAsset<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Resource, Default, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct PlayerData(pub HashMap<u64, Player>);

#[derive(Debug, Resource, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct ConnectedClients(pub HashSet<u64>);

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub color: [u8; 3],
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, PartialEq, Deref, DerefMut)]
pub struct Owner(pub u64);

#[derive(Component, Clone, Copy, Reflect, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Cursor {
    pub position: Vec2,
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum DeselectMessage {
    Everything,
    Entity(Entity),
}

impl MapEntities for DeselectMessage {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        match self {
            DeselectMessage::Entity(entity) => *entity = entity_mapper.map_entity(*entity),
            DeselectMessage::Everything => (),
        }
    }
}

impl Linear for Cursor {
    fn lerp(start: &Self, other: &Self, t: f32) -> Self {
        Cursor {
            position: (1.0 - t) * start.position + t * other.position,
        }
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
