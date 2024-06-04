use crate::prelude::*;
use client::Interpolated;
use lightyear::prelude::*;
use std::time::Duration;

pub const TICKRATE: f64 = 60.0;
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

        app.add_plugins(lightyear::shared::plugin::SharedPlugin { config })
            .add_systems(Update, (
                update_token_position,
                update_replicated_cursor_position,
            )
        );
    }
}

fn update_token_position(
    mut tokens: Query<(&mut Transform, &Token), Or<(With<Replicated>, With<Interpolated>)>>,
    time: Res<Time>,
) {
    for (mut transform, token) in tokens.iter_mut() {
        transform.translation = Vec2::lerp(
            transform.translation.xy(),
            token.position,
            (1.0 - 0.000000001f64.powf(time.delta_seconds_f64())) as f32,
        )
        .extend(token.layer);
    }
}

fn update_replicated_cursor_position(
    mut cursors: Query<(&mut Transform, &Cursor), Or<(With<Replicated>, With<Interpolated>)>>,
    time: Res<Time>,
) {
    for (mut transform, cursor) in cursors.iter_mut() {
        transform.translation = Vec2::lerp(
            transform.translation.xy(),
            cursor.position,
            (1.0 - 0.000000001f64.powf(time.delta_seconds_f64())) as f32,
        )
        .extend(50.0);
    }
}