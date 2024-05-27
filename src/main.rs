use bevy::prelude::*;
use bevy_inspector_egui::quick::*;

fn main() {
	App::default()
		.add_plugins((DefaultPlugins, WorldInspectorPlugin::default()))
		.run();
}
