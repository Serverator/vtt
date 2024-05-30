#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{log::LogPlugin, prelude::*};
use bevy_infinite_grid::InfiniteGridPlugin;
use bevy_mod_mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};
use bevy_mod_picking::DefaultPickingPlugins;
use image::imageops::FilterType;

mod dice;
mod input;
mod networking;
mod tabletop;
mod terminal;
mod windows;

fn main() {
    let mut args = std::env::args();
    let is_headless = args.any(|arg| arg == "--headless");

    let mut app = App::new();

    if is_headless {
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), LogPlugin::default()));
    } else {
        app.add_plugins((
            DefaultPlugins,
            MipmapGeneratorPlugin,
            DefaultPickingPlugins,
            InfiniteGridPlugin,
            input::InputPlugin,
            tabletop::TabletopPlugin,
            dice::DicePlugin,
            windows::WindowPlugin,
            bevy_egui::EguiPlugin,
        ))
        .insert_resource(MipmapGeneratorSettings {
            anisotropic_filtering: 16,
            filter_type: FilterType::Triangle,
            minimum_mip_resolution: 64,
        })
        .add_systems(Update, generate_mipmaps::<StandardMaterial>);

        #[cfg(debug_assertions)]
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }

    app.add_plugins((
        terminal::TerminalPlugin,
        networking::NetworkingPlugin {
            headless: is_headless,
        },
    ));

    app.run();
}

pub mod prelude {
    pub use bevy::prelude::FloatExt;
    pub use bevy::prelude::*;
    pub use bevy_egui::egui;
    pub use bevy_mod_picking::prelude::*;
    pub use serde::{Deserialize, Serialize};

    pub use crate::networking::protocol::*;
    pub use crate::networking::shared::DEFAULT_PORT;
}
