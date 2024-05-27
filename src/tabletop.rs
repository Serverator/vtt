use bevy::{input::mouse::MouseWheel, window::PrimaryWindow};
use bevy_egui::EguiContext;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridSettings};
use picking_core::PickingPluginsSettings;
use pointer::InputMove;

use crate::{input::{CursorPosition, OverUI}, prelude::*};

pub struct TabletopPlugin;
impl Plugin for TabletopPlugin {
    fn build(&self, app: &mut App) {
        app .insert_resource(Msaa::Sample4)
            .register_type::<Moving>()
            .add_systems(Startup, spawn_tabletop)
            .add_systems(PreUpdate, update_picking.after(crate::input::update_over_ui))
            .add_systems(Update, ((init_move_tokens, move_tokens).chain(), move_tabletop, zoom_tabletop));
    }
}

#[derive(Component)]
struct Token;

#[derive(Component, Reflect, Clone, Copy, Default)]
struct Moving {
    start_pos: Vec2,
    delta: Vec2,
}

fn update_picking(
    mut picking_settings: ResMut<PickingPluginsSettings>,
    movable: Query<&Moving>,
    over_ui: Res<OverUI>,
) {
    picking_settings.is_enabled = !**over_ui || !movable.is_empty();
}

fn init_move_tokens(
    mut moving_targets: Query<(&Transform, &mut Moving), Added<Moving>>,
) {
    for (transform, mut movement) in moving_targets.iter_mut() {
        movement.start_pos = transform.translation.xy();
    }
}

fn move_tokens(
    mut moving_targets: Query<(&mut Transform, &mut Moving), With<Moving>>,
    mut camera: Query<&mut Projection, With<Camera>>,
    mut mouse_motion: EventReader<InputMove>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let mouse_motion = mouse_motion.read().fold(Vec2::ZERO, |acc, x| {
         acc + x.delta
    });

    if mouse_input.pressed(MouseButton::Middle) {
        return;
    }

    let mut projection = camera.single_mut();

    let Projection::Orthographic(projection) = projection.as_mut() else {
        return;
    };

    for (mut transform, mut movement) in moving_targets.iter_mut() {
        movement.delta.x += mouse_motion.x * projection.scale;
        movement.delta.y -= mouse_motion.y * projection.scale;
        let mut new_pos = movement.start_pos + movement.delta;
        if !key_input.pressed(KeyCode::ShiftLeft) && !key_input.pressed(KeyCode::ShiftRight) {
            new_pos = (new_pos * 2.0).round() / 2.0;
        }
        transform.translation = new_pos.extend(transform.translation.z);
    }
}

fn move_tabletop(
    mut camera: Query<(&mut Transform, &Projection), With<Camera>>,
    input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<InputMove>,
) {
    let mouse_motion = mouse_motion.read().fold(Vec2::ZERO, |acc, x| {
         acc + x.delta
    });

    if !input.pressed(MouseButton::Middle) {
        return;
    }

    let  (mut transform, projection) = camera.single_mut();

    let Projection::Orthographic(projection) = projection else {
        return;
    };

    transform.translation.x += mouse_motion.x * -projection.scale;
    transform.translation.y += mouse_motion.y * projection.scale;
}

#[derive(Clone, Copy, Deref, DerefMut)]
pub struct ZoomLevel(f32);
impl Default for ZoomLevel {
	fn default() -> Self {
		Self(0.5)
	}
}

fn zoom_tabletop(
    mut camera: Query<(&mut Projection, &mut Transform), With<Camera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    egui: Query<&EguiContext>,
    cursor_pos: Res<CursorPosition>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut zoom_level: Local<ZoomLevel>,
    mut skip_first: Local<bool>,
) {
    let window = window.single();
    let window_size = Vec2::new(window.resolution.physical_width() as f32, window.resolution.physical_height() as f32);

    if egui.single().get().is_pointer_over_area() {
        return;
    }

    let mouse_wheel = mouse_wheel.read().fold(0.0, |acc, wheel| {
        acc + wheel.y
    });

    let (mut projection, mut transform) = camera.single_mut();

    let Projection::Orthographic(projection) = projection.as_mut() else {
        return;
    };

    let mut pos = cursor_pos.position;
    pos.x =  pos.x - window_size.x / 2.0;
    pos.y = -pos.y + window_size.y / 2.0;



    let zoom_before = projection.scale;
    zoom_level.0 = f32::clamp(zoom_level.0 - mouse_wheel * 0.04, 0.0, 1.0);
    projection.scale = f32::lerp(0.002, 0.1, zoom_level.powi(2));
    let zoom_delta = zoom_before - projection.scale;

    // Fix camera moving when table loads
    if !*skip_first {
        *skip_first = true;
        return;
    }

    transform.translation += pos.extend(0.0) * zoom_delta;

}

fn spawn_tabletop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
   	commands.spawn((
		Name::new("Grid"),
		InfiniteGridBundle {
			settings: InfiniteGridSettings {
			    x_axis_color: Color::BLACK,
				z_axis_color: Color::BLACK,
				major_line_color: Color::BLACK,
				minor_line_color: Color::BLACK,
				..default()
			},
			transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
			     .with_translation(Vec3::new(0.0, 0.0, 10.0)),
			..default()
		},
	));

    let bg_image = materials.add(StandardMaterial {
        unlit: true,
        base_color_texture: Some(asset_server.load("bg.jpg")),
        ..default()
    });

    let token_image = materials.add(StandardMaterial {
        unlit: true,
        base_color_texture: Some(asset_server.load("token.png")),
        ..default()
    });

    let quad = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));

    commands.spawn((
        Name::new("Map background"),
        PbrBundle {
            transform: Transform::from_scale(Vec3::new(32.0, 32.0, 1.0)),
            mesh: quad.clone(),
            material: bg_image,
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Token"),
        PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0).with_scale(Vec3::new(0.95, 0.95, 1.0)),
            mesh: quad.clone(),
            material: token_image,
            ..default()
        },
        Token,
        On::<Pointer<DragStart>>::target_commands_mut(|input, commands| {
            if input.button != PointerButton::Primary {
                return;
            }
            commands.insert(Moving::default());
        }),
        On::<Pointer<DragEnd>>::target_commands_mut(|input, commands| {
            if input.button != PointerButton::Primary {
                return;
            }
            commands.remove::<Moving>();
        }),
    ));

    commands.spawn((
        Name::new("Topdown camera"),
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 200.0),
            projection: Projection::Orthographic( OrthographicProjection {
                scale: 0.027,
                ..default()
            }),
            ..default()
        },
    ));
}
