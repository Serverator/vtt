use crate::prelude::*;
use lightyear::prelude::*;
use std::time::Duration;

pub const TICKRATE: f64 = 64.0;
pub const DEFAULT_PORT: u16 = 27007;

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        let config = SharedConfig {
            mode: Mode::HostServer,
            tick: TickConfig {
                tick_duration: Duration::from_secs_f64(1.0 / TICKRATE),
            },
            ..default()
        };

        app.add_plugins(lightyear::shared::plugin::SharedPlugin { config });
    }
}
