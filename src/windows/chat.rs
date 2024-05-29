use crate::prelude::*;
use bevy_egui::EguiContext;
use egui::*;
use lightyear::prelude::client::*;

pub struct ChatWindowPlugin;
impl Plugin for ChatWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (recieve_message, display_window).chain());

        app.init_resource::<ChatHistory>();
        // Create window
        app.world
            .spawn((Name::new("Chat Window"), ChatWindow::default()));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ChatHistory(pub Vec<ChatMessage>);

#[derive(Component, Debug, Default, Clone)]
pub struct ChatWindow {
    pub input: String,
}

fn recieve_message(
    mut messages: EventReader<MessageEvent<ChatMessage>>,
    mut chat_history: ResMut<ChatHistory>,
) {
    for message in messages.read() {
        chat_history.push(message.message.clone())
    }
}

fn display_window(
    mut egui_context: Query<&mut EguiContext>,
    mut chat_window: Query<(Entity, &mut ChatWindow)>,
    mut connection: ResMut<ConnectionManager>,
    chat_history: Res<ChatHistory>,
    player_list: Res<PlayerData>,
    client_state: Res<State<NetworkingState>>,
) {
    let (entity, mut chat_window) = chat_window.single_mut();
    let mut egui_context = egui_context.single_mut();

    let window = egui::Window::new("Chat window")
        .id(egui::Id::new(entity))
        .enabled(true)
        .collapsible(true);

    window.show(egui_context.get_mut(), |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(ui.available_height() - 50.0)
            .show_rows(
                ui,
                ui.text_style_height(&egui::style::TextStyle::Body),
                chat_history.len(),
                |ui, row_range| {
                    for row in row_range {
                        match &chat_history[row] {
                            ChatMessage::Message(id, message) => {
                                let player = player_list.get(id).cloned().unwrap_or_default();
                                let color = Color32::from_rgb(
                                    player.color[0],
                                    player.color[1],
                                    player.color[2],
                                );
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, format!("{}:", player.name));
                                    ui.colored_label(Color32::WHITE, message);
                                });
                            }
                            ChatMessage::Connected(id) => {
                                let player = player_list.get(id).cloned().unwrap_or_default();
                                ui.colored_label(
                                    Color32::YELLOW,
                                    format!("{} joined the game", player.name,),
                                );
                            }
                            ChatMessage::Disconnected(id) => {
                                let player = player_list.get(id).cloned().unwrap_or_default();
                                ui.colored_label(
                                    Color32::YELLOW,
                                    format!("{} left the game", player.name,),
                                );
                            }
                        }
                    }
                },
            );

        // Chat input and send button
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let has_text = !chat_window.input.trim().is_empty();

                let connected = matches!(client_state.get(), NetworkingState::Connected);

                let button = egui::Button::new("Send");
                let button_response = ui.add_enabled(has_text && connected, button);

                let text_edit = ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut chat_window.input),
                );

                if connected
                    && (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        || button_response.clicked())
                {
                    let message = SendMessage(String::from(chat_window.input.trim()));
                    _ = connection.send_message::<UnorderedReliableChannel, _>(&message);
                    chat_window.input.clear();
                }
            });
        });
    });
}
