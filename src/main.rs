use bevy::{log::LogPlugin, prelude::*};
use bevy_inspector_egui::quick::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_infinite_grid::InfiniteGridPlugin;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, MipmapGeneratorSettings, generate_mipmaps};
use image::imageops::FilterType;

mod tabletop;
mod input;
mod dice;
mod networking;
mod terminal;

fn main() {
	
	let mut args = std::env::args();
	let is_headless = args.any(|arg| arg == "--headless");

	let mut app = App::new();

	if is_headless {
		app.add_plugins((
			MinimalPlugins, 
			AssetPlugin::default(),
			LogPlugin::default()
		));
	} else {
		app.add_plugins((
			DefaultPlugins,
			MipmapGeneratorPlugin,
			WorldInspectorPlugin::default(),
			DefaultPickingPlugins,
			InfiniteGridPlugin,
			input::InputPlugin,
			tabletop::TabletopPlugin,
			dice::DicePlugin,
		));

		#[cfg(debug_assertions)]
		app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
	}

	app.add_plugins((
		terminal::TerminalPlugin,
		networking::NetworkingPlugin { headless: is_headless },
	));

	app.run();

	App::default()
		.add_plugins((
		    DefaultPlugins,
			MipmapGeneratorPlugin,
			WorldInspectorPlugin::default(),
			DefaultPickingPlugins,
			InfiniteGridPlugin,
			input::InputPlugin,
			tabletop::TabletopPlugin,
			dice::DicePlugin,
		))
		.insert_resource(MipmapGeneratorSettings {
		  anisotropic_filtering: 16,
				filter_type: FilterType::Triangle,
				minimum_mip_resolution: 64,
		})
		.add_systems(Update, generate_mipmaps::<StandardMaterial>)
		.run();
}

pub mod prelude {
	pub use bevy::prelude::*;
	pub use serde::{Serialize, Deserialize};
	pub use bevy_mod_picking::prelude::*;
	pub use bevy::prelude::FloatExt;
	pub use crate::networking::protocol::*;
}
