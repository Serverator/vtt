use std::net::{Ipv4Addr, SocketAddrV4};

use crate::{input::CursorPosition, prelude::*, tabletop::TopdownCamera};
use client::*;
use lightyear::{connection::netcode::PRIVATE_KEY_BYTES, prelude::*};
use rand::RngCore;
use super::{asset_sharing::RequestAssetMessage, shared::DEFAULT_PORT};

pub struct ClientPlugin {
    pub headless: bool,
}
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {

        let client_id = rand::thread_rng().next_u64();

        let config = ClientConfig {
            net: NetConfig::Netcode {
                auth: Authentication::Manual {
                    server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::new(127, 0, 0, 1),
                        DEFAULT_PORT,
                    )),
                    client_id,
                    private_key: [0; PRIVATE_KEY_BYTES],
                    protocol_id: 0,
                },
                config: NetcodeConfig::default(),
                io: IoConfig {
                    transport: ClientTransport::WebTransportClient {
                        client_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
                            Ipv4Addr::new(0, 0, 0, 0),
                            0,
                        )),
                        server_addr: std::net::SocketAddr::V4(SocketAddrV4::new(
                            Ipv4Addr::new(127, 0, 0, 1),
                            DEFAULT_PORT,
                        )),
                    },
                    compression: CompressionConfig::Zstd { level: 3 },
                    ..default()
                },
            },

            ..default()
        };

        app.add_systems(OnEnter(NetworkingState::Connected), send_player_info);
        app.insert_resource(ClientId(client_id));
        //app.add_systems(Startup, connect);

        if !self.headless {
            app .add_systems(Startup, spawn_local_cursor)
                .add_systems(Update, (
                    update_local_cursor_position,
                    init_replicated_cursors,
                    init_replicated_tokens,
                    update_replicated_cursor_color,
                ).run_if(in_state(NetworkingState::Connected)),
            );
        }

        app.add_plugins(ClientPlugins::new(config));
        app.insert_resource(Player {
            name: String::from("Player"),
            color: [255; 3],
        });

        app.add_systems(Update, recieve_message);
    }
}

#[derive(Resource, Clone, Copy, Deref, DerefMut)]
pub struct ClientId(pub u64);

fn send_player_info(mut connection: ResMut<ConnectionManager>, player: Res<Player>) {
    _ = connection.send_message::<UnorderedReliable, Player>(&player);
}

fn recieve_message(mut messages: EventReader<MessageEvent<ChatMessage>>) {
    for message in messages.read() {
        match &message.message {
            ChatMessage::Message(client, message) => info!("{client}: {message}"),
            ChatMessage::Connected(client) => info!("Client {client} connected"),
            ChatMessage::Disconnected(client) => info!("Client {client} disconnected"),
        }
    }
}

fn spawn_local_cursor(mut commands: Commands) {
    commands.spawn((
        Name::new("Cursor"),
        Cursor::default(),
        client::Replicate::default(),
    ));
}

fn update_local_cursor_position(
    mut local_cursor: Query<
        &mut Cursor,
        (
            Without<Replicated>,
            Without<Confirmed>,
            Without<Interpolated>,
        ),
    >,
    camera: Query<(&Camera, &GlobalTransform), With<TopdownCamera>>,
    cursor_pos: Res<CursorPosition>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !cursor_pos.is_changed() {
        return;
    }

    if mouse.pressed(MouseButton::Middle) {
        return;
    }

    let (camera, camera_transform) = camera.single();
    let mut local_cursor = local_cursor.single_mut();

    if let Some(position) = camera.viewport_to_world_2d(camera_transform, cursor_pos.position) {
        let desired_pos = position + Vec2::new(0.25, -0.25);
        local_cursor.position = desired_pos;
    }
}



fn init_replicated_tokens(
    mut commands: Commands,
    tokens: Query<(Entity, &Token, &SharedAsset<Image>), Or<(Added<Replicated>, Added<Interpolated>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    image_assets: Res<Assets<Image>>,
    mut connection: ResMut<ConnectionManager>,
) {

    for (entity, token, shared_image) in tokens.iter() {

        if image_assets.get(shared_image.id).is_none() {
            _ = connection.send_message::<UnorderedReliable, _>(&RequestAssetMessage::<Image>::new(shared_image.id));
        }

        let image = Handle::<Image>::weak_from_u128(shared_image.as_u128());
        let token_material = materials.add(StandardMaterial {
            unlit: true,
            base_color_texture: Some(image),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let quad = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));

        commands.entity(entity).insert((
            Name::new("Token"),
            PbrBundle {
                transform: Transform::from_translation(token.position.extend(token.layer)),
                mesh: quad.clone(),
                material: token_material,
                ..default()
            },
        ));
    }
}

fn init_replicated_cursors(
    mut commands: Commands,
    cursors: Query<(Entity, &Owner, &Cursor), Or<(Added<Replicated>, Added<Interpolated>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    player_data: Res<PlayerData>,
) {
    if cursors.is_empty() {
        return;
    }

    let quad = meshes.add(Mesh::from(Rectangle::new(0.5, 0.5)));
    let texture = asset_server.load("cursor.png");

    for (entity, owner, cursor) in cursors.iter() {
        let color = player_data
            .get(&owner.0)
            .map(|x| x.color)
            .unwrap_or([255; 3]);

        let material = materials.add(StandardMaterial {
            unlit: true,
            base_color_texture: Some(texture.clone()),
            base_color: Color::rgb_u8(color[0], color[1], color[2]),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        let mut entity = commands.entity(entity);

        entity.insert((
            Name::new(format!("Cursor (ID:{})", owner.0)),
            PbrBundle {
                material,
                mesh: quad.clone(),
                transform: Transform::from_translation(cursor.position.extend(50.0)),
                ..default()
            },
        ));
    }
}

fn update_replicated_cursor_color(
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_data: Res<PlayerData>,
    cursors: Query<(&Handle<StandardMaterial>, &Owner)>,
) {
    if !player_data.is_changed() {
        return;
    }

    for (cursor, owner) in cursors.iter() {
        let color = player_data
            .get(&owner.0)
            .map(|x| x.color)
            .unwrap_or([255; 3]);

        if let Some(material) = materials.get_mut(cursor) {
            material.base_color = Color::rgb_u8(color[0], color[1], color[2]);
        }
    }
}
