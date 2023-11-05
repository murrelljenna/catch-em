use std::net::{SocketAddr, UdpSocket};

use bevy::{log::LogPlugin, prelude::*};
use bevy_fps_controller::controller::FpsControllerPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::RapierConfiguration;
use crate::networking::{ClientPlugin, NetworkEvent};
use crate::networking::systems::{auto_heartbeat_system, Socket, SocketAddress};
use crate::{display_text, manage_cursor, respawn, scene_colliders, setup};
use crate::networking::message::{Message, serialize};
use crate::networking::message::Message::{NetworkInput, SpawnPlayer};
use crate::networking::send_input::send_player_input;
use crate::networking::send_player_position::send_player_position;
use crate::player::spawn_player;

pub fn main(socket_addr: String) {
    let remote_addr: SocketAddr = "127.0.0.1:8080".parse().expect("could not parse addr");
    println!("{}", socket_addr);
    let socket = UdpSocket::bind(socket_addr).expect("could not bind socket");
    println!("{}", remote_addr.is_ipv4());
    socket
        .connect(remote_addr)
        .expect("could not connect to server");
    socket
        .set_nonblocking(true)
        .expect("could not set socket to be nonblocking");

    App::new()
        .insert_resource(SocketAddress(remote_addr))
        .insert_resource(Socket(socket))
        .add_plugins(ClientPlugin)
        .add_systems(Update, connection_handler)

        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (manage_cursor, scene_colliders, display_text, respawn))
        .add_systems(Update, wait_for_spawn_player)
        .add_systems(Update, auto_heartbeat_system)
        .add_systems(Update, send_player_position)
        .run();
}

fn wait_for_spawn_player(mut commands: Commands, mut messages: EventReader<Message>, ) {
    for message in messages.iter() {
        match message {
            SpawnPlayer(.., pos) => spawn_player(*pos, &mut commands),
                _ => ()
        }
    }
}

fn connection_handler(mut events: EventReader<NetworkEvent>, mut messages: EventWriter<Message>) {
    for event in events.iter() {
        match event {
            NetworkEvent::RawMessage(_, msg) => {
                info!("server sent a message: {:?}", msg);
                messages.send(*msg);
            }
            NetworkEvent::SendError(err, msg) => {
                error!(
                    "NetworkEvent::SendError (payload [{:?}]): {:?}",
                    msg.payload, err
                );
            }
            NetworkEvent::RecvError(err) => {
                error!("NetworkEvent::RecvError: {:?}", err);
            }
            // discard irrelevant events
            _ => {}
        }
    }
}