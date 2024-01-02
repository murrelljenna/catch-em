use bevy::app::AppExit;
use std::net::{SocketAddr, UdpSocket};

use crate::networking::components::NetworkObjectType;
use crate::networking::components::{NetworkObject, NetworkTransform};
use crate::networking::handshake::{listen_handshake_events, ConnectionStatus};
use crate::networking::message::Message;
use crate::networking::message::Message::{Despawn, PlayerPosition, Spawn};
use crate::networking::player::PlayerId;

use crate::game::entities::{spawn_player, spawn_player_facade};
use crate::networking::packet_systems::{auto_heartbeat_system, Socket, SocketAddress};
use crate::networking::send_player_position::send_player_position;
use crate::networking::{ClientPlugin, NetworkEvent, Transport};
use crate::{display_text, manage_cursor, respawn, scene_colliders, setup};
use bevy::prelude::*;
use bevy_fps_controller::controller::FpsControllerPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::RapierConfiguration;

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
        .add_systems(
            Update,
            (manage_cursor, scene_colliders, display_text, respawn),
        )
        .add_systems(Update, auto_heartbeat_system)
        .add_systems(Update, send_player_position)
        .add_systems(Update, listen_events)
        .add_systems(Update, NetworkTransform::sync_network_transforms)
        .run();
}

fn spawn_network_object(
    object_type: &NetworkObjectType,
    object_id: u8,
    id: PlayerId,
    pos: Vec3,
    mut commands: &mut Commands,
) {
    println!("Spawning net object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player(id, object_id, pos, &mut commands);
        }
    }
}

fn spawn_network_facade_object(
    object_type: &NetworkObjectType,
    object_id: u8,
    id: PlayerId,
    pos: Vec3,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    println!("Spawning facade object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player_facade(
                id,
                object_id,
                pos,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    }
}

fn listen_game_events(
    mut commands: Commands,
    mut messages: EventReader<Message>,
    mut local_player_id: ResMut<PlayerId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut networked_entities: Query<(&NetworkObject, Entity)>,
    mut networked_objects: Query<(&NetworkObject, &mut NetworkTransform)>,
    timer: Res<Time>,
) {
    for message in messages.iter() {
        println!("{:?}", message);
        match message {
            Spawn(id, pos, object_type, object_id) if (*id == *local_player_id) => {
                spawn_network_object(object_type, *object_id, *id, *pos, &mut commands);
                *local_player_id = *id;
            }
            Spawn(id, pos, object_type, object_id) => spawn_network_facade_object(
                object_type,
                *object_id,
                *id,
                *pos,
                &mut commands,
                &mut meshes,
                &mut materials,
            ),
            PlayerPosition(received_player_id, pos, _object_id) => {
                NetworkTransform::update_last_pos(received_player_id, pos, &mut networked_objects);
            }
            Despawn(_, object_id) => {
                for (object, entity) in networked_entities.iter_mut() {
                    if (object.id == *object_id) {
                        commands.entity(entity).despawn();
                    }
                }
            }

            _ => (),
        }
    }
}

fn listen_events(
    socket: Res<Socket>,
    commands: Commands,
    transport: ResMut<Transport>,
    messages: EventReader<Message>,
    local_player_id: ResMut<PlayerId>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    networked_objects: Query<(&NetworkObject, &mut NetworkTransform)>,
    mut networked_entities: Query<(&NetworkObject, Entity)>,
    timer: Res<Time>,
    connection_status: ResMut<ConnectionStatus>,
) {
    match *connection_status {
        ConnectionStatus::Initial => listen_handshake_events(
            messages,
            socket,
            transport,
            local_player_id,
            connection_status,
        ),
        _ => listen_game_events(
            commands,
            messages,
            local_player_id,
            meshes,
            materials,
            networked_entities,
            networked_objects,
            timer,
        ),
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
