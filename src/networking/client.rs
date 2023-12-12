use std::net::{SocketAddr, UdpSocket};

use bevy::{log::LogPlugin, prelude::*};
use bevy_fps_controller::controller::FpsControllerPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::RapierConfiguration;
use serde::Serialize;
use crate::networking::{ClientPlugin, NetworkEvent, Transport};
use crate::networking::systems::{auto_heartbeat_system, Socket, SocketAddress};
use crate::{display_text, manage_cursor, respawn, scene_colliders, setup};
use crate::networking::handshake::{ConnectionStatus, listen_handshake_events};
use crate::networking::message::{Message, serialize};
use crate::networking::message::Message::{NetworkInput, Spawn, PlayerPosition, ServerAcknowledgement};
use crate::networking::player::{NetworkObject, NetworkObjectType, PlayerId};
use crate::networking::send_input::send_player_input;
use crate::networking::send_player_position::send_player_position;
use crate::player::{spawn_player, spawn_player_facade};

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
        .insert_resource(ConnectionStatus::Initial)
        .add_plugins(ClientPlugin)
        .add_systems(Update, connection_handler)

        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .insert_resource(PlayerId(0))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (manage_cursor, scene_colliders, display_text, respawn))
        .add_systems(Update, auto_heartbeat_system)
        .add_systems(Update, send_player_position)
        .add_systems(Update, listen_events)
        .run();
}

fn spawn_network_object(object_type: &NetworkObjectType, object_id: u8, id: PlayerId, pos: Vec3, mut commands: &mut Commands) {
    println!("Spawning net object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player(id, object_id, pos, &mut commands);
        }
    }
}

fn spawn_network_facade_object(
    object_type: &NetworkObjectType, object_id: u8, id: PlayerId, pos: Vec3, mut commands: &mut Commands, mut meshes: &mut ResMut<Assets<Mesh>>, mut materials: &mut ResMut<Assets<StandardMaterial>>
) {
    println!("Spawning facade object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player_facade(id, object_id, pos, &mut commands, &mut meshes, &mut materials);
        }
    }
}

fn listen_game_events(mut commands: Commands, mut messages: EventReader<Message>, mut local_player_id: ResMut<PlayerId>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut networked_objects: Query<(&NetworkObject, &mut Transform)>, timer: Res<Time>) {
    for message in messages.iter() {
        println!("{:?}", message);
        match message {
            Spawn(id, pos, object_type, object_id) if (*id == *local_player_id) => {
                spawn_network_object(object_type, *object_id, *id, *pos, &mut commands);
                *local_player_id = *id;
            },
            Spawn(id, pos, object_type, object_id) => spawn_network_facade_object(object_type, *object_id, *id, *pos, &mut commands, &mut meshes, &mut materials),
            PlayerPosition(received_player_id, pos, object_id) => {
                println!("Received player pos message");
                for (networked_object, mut transform) in networked_objects.iter_mut() {
                    println!("Iterating over net objects.");
                    if networked_object.owner == *received_player_id {
                        let incremental_adjust = 0.8 * timer.delta_seconds();
                        let old_translation = transform.translation;
                        transform.translation = old_translation.lerp(*pos, incremental_adjust);
                        println!("Found the player's object. Updating pos.")
                    }
                }
            },

            _ => ()
        }
    }
}

fn listen_events(socket: Res<Socket>, mut commands: Commands, mut transport: ResMut<Transport>, mut messages: EventReader<Message>, mut local_player_id: ResMut<PlayerId>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut networked_objects: Query<(&NetworkObject, &mut Transform)>, timer: Res<Time>, mut connection_status: ResMut<ConnectionStatus>) {
    match *connection_status {
        ConnectionStatus::Initial => listen_handshake_events(messages, socket, transport, local_player_id, connection_status),
        _ => listen_game_events(commands, messages, local_player_id, meshes, materials, networked_objects, timer)
    }
}

//fn update_player_position(mut commands: Commands, mut messages: EventReader<Message>)



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