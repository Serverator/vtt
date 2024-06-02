use std::marker::PhantomData;

use crate::prelude::*;
use bevy::{
    ecs::entity::MapEntities, reflect::{serde::{ReflectSerializer, TypedReflectDeserializer}, TypeRegistration, TypeRegistry, Typed}, utils::{HashMap, HashSet}
};
use lightyear::prelude::*;
use serde::de::DeserializeSeed;
use uuid::Uuid;

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

        app.add_shared_reflect_asset::<Mesh>();
        app.add_shared_reflect_asset::<StandardMaterial>();
        app.add_shared_reflect_asset::<Image>();

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

pub trait SharedAssetExt {
    fn add_shared_asset<T: Asset + Message>(&mut self) -> &mut Self;

    fn add_shared_reflect_asset<T: Asset + Reflect>(&mut self) -> &mut Self;
}

impl SharedAssetExt for App {
    fn add_shared_asset<T: Asset + Message>(&mut self) -> &mut Self {
        self.init_resource::<SharedAssets<T>>();
        self.add_message::<RequestAssetMessage<T>>(ChannelDirection::Bidirectional);
        self.add_message::<SendAssetMessage<T>>(ChannelDirection::Bidirectional);

        self
    }

    fn add_shared_reflect_asset<T: Asset + Reflect>(&mut self) -> &mut Self {
        self.init_resource::<SharedAssets<T>>();
        self.add_message::<RequestAssetMessage<T>>(ChannelDirection::Bidirectional);
        self.add_message::<SendReflectAssetMessage<T>>(ChannelDirection::Bidirectional);

        self
    }
}

#[derive(Channel)]
pub struct UnorderedReliable;

#[derive(Channel)]
pub struct SequencedUnreliable;

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub position: Vec2,
    pub layer: f32,
}

pub enum SharedAssetId {
    Uuid(Uuid),
    Name(String),
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RequestAssetMessage<T> {
    id: Uuid,
    _spooky: PhantomData<T>,
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SendAssetMessage<T> {
    id: Uuid,
    data: T,
}

#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendReflectAssetMessage<T> {
    id: Uuid,
    // Pre-encoded by `bincode`
    data: Vec<u8>,
    _spooky: PhantomData<T>
}

impl<T: Reflect + Typed> SendReflectAssetMessage<T> {
    pub fn new(uuid: Uuid, data: &T, registry: &TypeRegistry) -> Result<Self, bincode::Error> {
        let serializer = ReflectSerializer::new(data, registry);
        let bytes = bincode::serialize(&serializer)?;

        Ok(Self {
            id: uuid,
            data: bytes,
            _spooky: PhantomData::default(),
        })
    }

    pub fn deserialize(&self, registry: &TypeRegistry) -> Result<T, bincode::Error> {
         let registration = TypeRegistration::of::<T>();
         let reflect_deserializer = TypedReflectDeserializer::new(&registration, registry);
         let mut deserializer = bincode::Deserializer::from_slice(&self.data[..], bincode::config::DefaultOptions::default());
         let reflect_value = reflect_deserializer.deserialize(&mut deserializer)?;
         Ok(*reflect_value.downcast::<T>().unwrap())
    }
}

#[derive(Resource)]
pub struct SharedAssets<T: Asset> {
    pub map: HashMap<uuid::Uuid, Handle<T>>,
}

// Derive macro for some reason refuses to impl Default
impl<T: Asset> Default for SharedAssets<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
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
