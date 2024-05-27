use bevy_egui::EguiContext;
use pointer::InputMove;
use crate::prelude::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app .init_resource::<OverUI>()
            .init_resource::<CursorPosition>()
            .register_type::<OverUI>()
            .register_type::<CursorPosition>()
            .add_systems(PreUpdate, (update_over_ui, update_cursor_position));
    }
}

#[derive(Default, Reflect, Resource, Deref, DerefMut)]
pub struct OverUI(pub bool);

#[derive(Resource, Reflect, Clone, Copy, Default)]
pub struct CursorPosition {
    pub position: Vec2,
}

pub fn update_cursor_position(
    mut mouse_motion: EventReader<InputMove>,
    mut mouse_pos: ResMut<CursorPosition>,
) {
    if let Some(pos) = mouse_motion.read().last() {
        mouse_pos.position = pos.location.position;
    }
}

pub fn update_over_ui(
    mut over_ui: ResMut<OverUI>,
    egui: Query<&EguiContext>,
) {
    over_ui.0 = egui.single().get().is_pointer_over_area();
}
