use bevy::prelude::*;
use bevy_inspector_egui::quick::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_infinite_grid::InfiniteGridPlugin;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, MipmapGeneratorSettings, generate_mipmaps};
use image::imageops::FilterType;

mod tabletop;
mod input;
mod dice;

fn main() {
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
		      ..default()
		})
		.add_systems(Update, generate_mipmaps::<StandardMaterial>)
		.run();
}

pub mod prelude {
	pub use bevy::prelude::*;
	pub use bevy_mod_picking::prelude::*;
}
