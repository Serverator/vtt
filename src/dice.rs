use bevy::render::{camera::RenderTarget, view::RenderLayers};
use pointer::InputMove;

use crate::prelude::*;

pub struct DicePlugin;
impl Plugin for DicePlugin {
	fn build(&self, app: &mut App) {
		app
          .add_systems(Startup, spawn_dummy_dices)
		  .add_systems(Update, (process_clicked, update_velocity))
		  .add_systems(Startup, spawn_dice_camera)
		  .register_type::<SelectorDiceVelocity>();
	}
}


#[derive(Component)]
pub struct DummyDice(pub u8);

#[derive(Component, Default)]
pub struct Grabbed {
    grab_time: f32,
}


#[derive(Component, Default, Reflect)]
pub struct SelectorDiceVelocity(pub Vec2);

pub fn spawn_dummy_dices(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mesh = meshes.add(Mesh::from(Cuboid::default()));
	let material = materials.add(StandardMaterial {
	   unlit: true,
		..default()
	});

	commands.spawn((
		Name::new("D6 selector dice"),
		PbrBundle {
			mesh,
			material,
			transform: Transform::from_xyz(0.0, -4.0, 0.0),
			..default()
		},
		DummyDice(6),
		RenderLayers::layer(1),
		SelectorDiceVelocity::default(),
        On::<Pointer<DragStart>>::target_commands_mut(|input, commands| {
            if input.button != PointerButton::Primary {
                return;
            }
            commands.insert(Grabbed::default());
        }),
        On::<Pointer<DragEnd>>::target_commands_mut(|input, commands| {
            if input.button != PointerButton::Primary {
                return;
            }
            commands.remove::<Grabbed>();
        }),
	));
}

pub fn update_velocity(
	mut dummy_q: Query<(&mut SelectorDiceVelocity, &mut Transform), With<DummyDice>>,
	time: Res<Time>,
) {
	for (mut velocity, mut transform) in dummy_q.iter_mut() {


		let rotation = transform.rotation;
		let vel_rot = Quat::from_euler(
			EulerRot::default(),
			velocity.0.x * time.delta_seconds(),
			velocity.0.y * time.delta_seconds(),
			0.0,
		);

		// I don't fucking know how it works, but it does.
		transform.rotation = rotation * (rotation.inverse() * vel_rot * rotation).normalize();
		velocity.0 = velocity
			.0.lerp(Vec2::ZERO, 1.0 - 0.2f32.powf(time.delta_seconds()));
	}
}


#[allow(clippy::too_many_arguments)]
pub fn process_clicked(
    mut commands: Commands,
	time: Res<Time>,
	mut mouse_motion: EventReader<InputMove>,
	mut grabbed_dice: Query<(Entity, &mut SelectorDiceVelocity, &mut Grabbed), With<DummyDice>>,
) {
    let motion: Vec2 = mouse_motion.read().map(|x| x.delta).sum();

    for (entity, mut velocity, mut grab_time) in grabbed_dice.iter_mut() {
        velocity.0 += motion * 0.08;
        grab_time.grab_time += time.delta_seconds();

        if grab_time.grab_time > 0.08 {
           commands.entity(entity).remove::<Grabbed>();
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DiceCamera;

fn spawn_dice_camera(
    mut commands: Commands
) {
	commands.spawn((
		Name::new("Light"),
		DirectionalLightBundle {
			directional_light: DirectionalLight {
				illuminance: 30000.0,
				..default()
			},
			transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 1.5, 3.3, -2.1)),
			..default()
		},
	));

	commands.insert_resource(AmbientLight {
		color: Color::ALICE_BLUE,
		brightness: 0.15,
	});

	commands.spawn((
		Name::new("Dice Camera"),
		Camera3dBundle {
			camera: Camera {
				order: 5,
				clear_color: ClearColorConfig::Custom(Color::rgba_u8(0, 0, 0, 0)),
				target: RenderTarget::Window(bevy::window::WindowRef::Primary),
				..default()
			},
			projection: Projection::Orthographic(OrthographicProjection {
				scale: 0.01,
				..default()
			}),
			transform: Transform::from_xyz(0.0, 0.0, 10.0),
			..default()
		},
		DiceCamera,
		RenderLayers::layer(1),
	));
}
