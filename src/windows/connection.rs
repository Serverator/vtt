use crate::{networking::shared::DEFAULT_PORT, prelude::*};
use bevy_egui::{
    egui::{self, Color32},
    EguiContext,
};
use client::{Authentication, ClientCommands, ClientConnection};
use lightyear::{
    connection::{client::NetClientDispatch, netcode::ClientState},
    prelude::*,
};
use server::ServerCommands;
use std::{
    borrow::Cow,
    net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs},
};

pub struct ConnectionWindowPlugin;
impl Plugin for ConnectionWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(client::NetworkingState::Connecting),
            update_connection_state,
        )
        .add_systems(Update, display_window);

        // Create window
        app.world
            .spawn((Name::new("Connection Window"), ConnectionWindow::default()));
    }
}

fn display_window(
    mut egui_context: Query<&mut EguiContext>,
    mut connection_window: Query<(Entity, &mut ConnectionWindow)>,
    mut connection: ResMut<client::ConnectionManager>,
    mut client_config: ResMut<client::ClientConfig>,
    mut player: ResMut<Player>,
    mut commands: Commands,
    server_state: Res<State<server::NetworkingState>>,
    client_state: Res<State<client::NetworkingState>>,
) {
    let (entity, mut connection_window) = connection_window.single_mut();
    let mut egui_context = egui_context.single_mut();

    let window = egui::Window::new("Connection window")
        .id(egui::Id::new(entity))
        .enabled(true)
        .collapsible(true);

    window.show(egui_context.get_mut(), |ui| {
        ui.set_max_width(260.0);

        let changed = ui
            .horizontal(|ui| {
                ui.label("Username");
                let name_str = &mut player.name;
                let text_edit = egui::TextEdit::singleline(name_str).char_limit(20);
                let text_edit = ui.add_sized(ui.available_size() - egui::Vec2::X * 50.0, text_edit);

                let color_picker_response = ui.color_edit_button_srgb(&mut player.color);

                text_edit.changed() || color_picker_response.changed()
            })
            .inner;

        if changed {
            _ = connection.send_message::<UnorderedReliableChannel, Player>(&player);
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("IP Adress");
            ui.text_edit_singleline(&mut connection_window.address_input);
        });

        ui.horizontal(|ui| {
            match client_state.get() {
                client::NetworkingState::Disconnected => {
                    if ui.button("Connect").clicked() {
                        let address = if connection_window.address_input.trim().is_empty() {
                            Some(std::net::SocketAddr::V4(SocketAddrV4::new(
                                Ipv4Addr::new(127, 0, 0, 1),
                                DEFAULT_PORT,
                            )))
                        } else if connection_window.address_input.contains(':') {
                            connection_window
                                .address_input
                                .trim()
                                .to_socket_addrs()
                                .ok()
                                .and_then(|mut x| x.find(|x| x.is_ipv4()))
                        } else {
                            format!(
                                "{}:{}",
                                &connection_window.address_input.trim(),
                                DEFAULT_PORT
                            )
                            .to_socket_addrs()
                            .ok()
                            .and_then(|mut x| x.find(|x| x.is_ipv4()))
                        };

                        if let Some(address) = address {
                            if let client::NetConfig::Netcode {
                                auth: Authentication::Manual { server_addr, .. },
                                ..
                            } = &mut client_config.net
                            {
                                *server_addr = address;
                            }

                            commands.connect_client();
                        } else {
                            connection_window.error = Cow::Borrowed("Incorrect IP adress");
                        }
                    }
                }
                client::NetworkingState::Connecting => {
                    ui.add_enabled_ui(false, |ui| ui.button("Connecting"));
                }
                client::NetworkingState::Connected => {
                    if ui.button("Disconnect").clicked() {
                        commands.disconnect_client();
                    }
                }
            };

            match server_state.get() {
                server::NetworkingState::Stopped => {
                    let enabled =
                        matches!(client_state.get(), client::NetworkingState::Disconnected);
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.button("Host").clicked() {
                            commands.start_server();
                        }
                    });
                }
                server::NetworkingState::Started => {
                    if ui.button("Stop hosting").clicked() {
                        commands.stop_server();
                    }
                }
            }
        });

        if !connection_window.error.is_empty() {
            ui.colored_label(Color32::RED, &*connection_window.error);
        }
    });
}

fn update_connection_state(
    client: Res<ClientConnection>,
    mut connection_window: Query<&mut ConnectionWindow>,
) {
    if let NetClientDispatch::Netcode(client) = &client.client {
        let mut connection_window = connection_window.single_mut();

        let state = client.client.state();
        match state {
            ClientState::ConnectTokenExpired => {
                connection_window.error = Cow::Borrowed("Connection token expired")
            }
            ClientState::ConnectionTimedOut
            | ClientState::ConnectionRequestTimedOut
            | ClientState::ChallengeResponseTimedOut => {
                connection_window.error = Cow::Borrowed("Connection timed out")
            }
            ClientState::ConnectionDenied => {
                connection_window.error = Cow::Borrowed("Connection denied")
            }
            _ => (),
        }
    }
}

#[derive(Component, Default)]
pub struct ConnectionWindow {
    address_input: String,
    error: Cow<'static, str>,
}
