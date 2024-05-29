use std::{
    collections::VecDeque,
    net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs},
    sync::Mutex,
};

use bevy::{ecs::event::ManualEventReader, utils::HashMap};
use client::{Authentication, ClientConfig, ConnectionManager};
use lightyear::prelude::*;

use crate::{networking::shared::DEFAULT_PORT, prelude::*};

#[derive(Event, Debug, Default, Deref, DerefMut, Clone)]
pub struct RawTerminalCommand(String);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CommandRegistry(HashMap<&'static str, Box<dyn Command>>);

trait AddTerminalCommand {
    fn add_command(&mut self, command_impl: impl Command + 'static) -> &mut Self;

    fn init_command<T: Command + FromWorld + 'static>(&mut self) -> &mut Self;
}

impl AddTerminalCommand for App {
    fn add_command(&mut self, command_impl: impl Command + 'static) -> &mut Self {
        let mut registry = self
            .world
            .get_resource_or_insert_with::<CommandRegistry>(Default::default);
        registry.insert(command_impl.stem(), Box::new(command_impl));
        self
    }

    fn init_command<T: Command + FromWorld + 'static>(&mut self) -> &mut Self {
        let default = T::from_world(&mut self.world);
        self.add_command(default)
    }
}

pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandRegistry>()
            .add_event::<RawTerminalCommand>()
            .init_command::<HelpCommand>()
            .init_command::<HostCommand>()
            .init_command::<SendCommand>()
            .init_command::<ConnectCommand>()
            .add_systems(Startup, spawn_stdin_reader)
            .add_systems(PreUpdate, (send_raw_event, process_raw_events));
    }
}

static RECIEVER: Mutex<VecDeque<RawTerminalCommand>> = Mutex::new(VecDeque::new());

fn spawn_stdin_reader() {
    let thread_pool = bevy::tasks::IoTaskPool::get();
    let task = thread_pool.spawn(async move {
        loop {
            let stdin = std::io::stdin();
            let mut string = String::new();
            _ = stdin.read_line(&mut string).unwrap();
            if string.trim().is_empty() {
                continue;
            }
            if string.ends_with('\n') {
                _ = string.pop();
            }
            if let Ok(mut lock) = RECIEVER.lock() {
                lock.push_back(RawTerminalCommand(string));
            };
        }
    });

    std::mem::forget(task);
}

fn send_raw_event(mut ev_writer: EventWriter<RawTerminalCommand>) {
    let Ok(mut lock) = RECIEVER.try_lock() else {
        return;
    };

    while let Some(command) = lock.pop_front() {
        ev_writer.send(command);
    }
}

#[reflect_trait]
pub trait Command: Send + Sync {
    fn stem(&self) -> &'static str;

    fn help_string(&self) -> &'static str;

    fn run_command(&mut self, args: &str, world: &mut World);
}

#[derive(Default)]
struct HelpCommand;

impl Command for HelpCommand {
    fn run_command(&mut self, _args: &str, world: &mut World) {
        let registry = world.resource::<CommandRegistry>();
        println!("List of all commands:");

        for command in registry.values() {
            println!("  {} - {}", command.stem(), command.help_string());
        }
    }

    fn stem(&self) -> &'static str {
        "help"
    }

    fn help_string(&self) -> &'static str {
        "Show this menu"
    }
}

#[derive(Default)]
struct SendCommand;

impl Command for SendCommand {
    fn run_command(&mut self, args: &str, world: &mut World) {
        let mut connection = world.get_resource_mut::<ConnectionManager>().unwrap();
        connection
            .send_message::<UnorderedReliableChannel, SendMessage>(&SendMessage(String::from(args)))
            .unwrap();
    }

    fn stem(&self) -> &'static str {
        "send"
    }

    fn help_string(&self) -> &'static str {
        "Sends a message to another server/client"
    }
}

#[derive(Default)]
struct HostCommand;

impl Command for HostCommand {
    fn run_command(&mut self, _args: &str, world: &mut World) {
        world.insert_resource(NextState::<server::NetworkingState>(Some(
            server::NetworkingState::Started,
        )))
    }

    fn stem(&self) -> &'static str {
        "host"
    }

    fn help_string(&self) -> &'static str {
        "Hosts server"
    }
}

#[derive(Default)]
struct ConnectCommand;

impl Command for ConnectCommand {
    fn run_command(&mut self, args: &str, world: &mut World) {
        let address = if args.trim().is_empty() {
            Some(std::net::SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 0, 1),
                DEFAULT_PORT,
            )))
        } else if args.contains(':') {
            args.trim()
                .to_socket_addrs()
                .ok()
                .and_then(|mut x| x.find(|x| x.is_ipv4()))
        } else {
            format!("{}:{}", &args.trim(), DEFAULT_PORT)
                .to_socket_addrs()
                .ok()
                .and_then(|mut x| x.find(|x| x.is_ipv4()))
        };

        println!("{:?}", address);

        if let Some(address) = address {
            let mut client_config = world.resource_mut::<ClientConfig>();
            if let client::NetConfig::Netcode {
                auth: Authentication::Manual { server_addr, .. },
                ..
            } = &mut client_config.net
            {
                *server_addr = address;
            }

            world.insert_resource(NextState::<client::NetworkingState>(Some(
                client::NetworkingState::Connecting,
            )));
        } else {
            error!("Incorrect IP adress");
        }
    }

    fn stem(&self) -> &'static str {
        "connect"
    }

    fn help_string(&self) -> &'static str {
        "Connects to another server"
    }
}

fn process_raw_events(
    world: &mut World,
    mut ev_reader: Local<ManualEventReader<RawTerminalCommand>>,
) {
    world.resource_scope::<Events<RawTerminalCommand>, ()>(|world, events| {
        for raw in ev_reader.read(&events) {
            let (command, args) = raw.split_once(' ').unwrap_or_else(|| (raw.as_str(), ""));
            let command = command.trim();

            // `HelpCommand`` needs `CommandRegistry` to be in scope, hence the unsafe
            // TODO: Make CommandRegistry not available when running command..?
            let mut command_registry: Mut<'static, CommandRegistry> =
                unsafe { std::mem::transmute(world.resource_mut::<CommandRegistry>()) };

            let Some(command) = command_registry.get_mut(command) else {
                error!(
                    "Command \"{}\" not found.\nType \"help\" to list all commands",
                    command
                );
                return;
            };

            command.run_command(args.trim(), world);
        }
    });
}
