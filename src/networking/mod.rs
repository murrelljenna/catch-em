pub mod components;
pub mod events;
pub mod handshake;
pub mod message;
pub mod packet_systems;
pub mod resources;
pub mod raw_message;
pub mod send_input;
pub mod send_player_position;

mod transport;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub use self::events::NetworkEvent;
pub use self::transport::Transport;

use bevy::prelude::*;
use crate::networking::components::{NetworkObject, NetworkTransform};
use crate::networking::handshake::{ConnectionStatus, listen_handshake_events};
use crate::networking::message::Message;
use crate::networking::message::Message::{Despawn, NetworkPosition, Spawn};
use crate::networking::packet_systems::{auto_heartbeat_system, Socket, SocketAddress, SocketLive};
use crate::networking::resources::{NetworkGame, PlayerId};
use crate::networking::send_player_position::sync_network_transforms;

/// Defines how many times a client automatically sends a heartbeat packet.
/// This should be no more than half of idle_timeout.
const DEFAULT_HEARTBEAT_TICK_RATE_SECS: f32 = 2.;
/// Defines how long the server will wait until it sends
/// NetworkEvent::Disconnected
const DEFAULT_IDLE_TIMEOUT_SECS: f32 = 5.;

#[derive(Resource)]
pub struct NetworkResource {
    // Hashmap of each live connection and their last known packet activity
    pub connections: HashMap<SocketAddr, Duration>,
    pub idle_timeout: Duration,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            connections: Default::default(),
            idle_timeout: Duration::from_secs_f32(DEFAULT_IDLE_TIMEOUT_SECS),
        }
    }
}

/// Label for network related systems.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum NetworkSystem {
    Receive,
    Send,
}

/// Label for server specific systems.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum ServerSystem {
    IdleTimeout,
}

/// Label for client specific systems.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum ClientSystem {
    Heartbeat,
}

pub struct ServerPlugin(pub String);

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let socket = UdpSocket::bind(self.0.as_str()).expect("could not bind socket");
        socket
            .set_nonblocking(true)
            .expect("could not set socket to be nonblocking");
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("could not set read timeout");

        app.insert_resource(NetworkResource::default())
            .insert_resource(transport::Transport::new())
            .add_event::<events::NetworkEvent>()
            .add_systems(Update, packet_systems::server_recv_packet_system)
            .add_systems(Update, packet_systems::send_packet_system)
            .add_systems(Update, packet_systems::idle_timeout_system).insert_resource(Socket(Box::new(SocketLive(socket))))
            .insert_resource(NetworkGame::default());
    }
}

#[derive(Resource)]
pub struct HeartbeatTimer(Timer);

pub struct ClientPlugin(pub String, pub String);

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let remote_addr: SocketAddr = self.0.parse().expect("could not parse addr");
        let socket = UdpSocket::bind(self.1.clone()).expect("could not bind socket");
        socket
            .connect(remote_addr)
            .expect("could not connect to server");
        socket
            .set_nonblocking(true)
            .expect("could not set socket to be nonblocking");

        app.insert_resource(transport::Transport::new())
            .insert_resource(HeartbeatTimer(Timer::from_seconds(
                DEFAULT_HEARTBEAT_TICK_RATE_SECS,
                Default::default(),
            )))
            .insert_resource(SocketAddress(remote_addr))
            .insert_resource(Socket(Box::new(SocketLive(socket))))
            .add_event::<events::NetworkEvent>()
            .add_event::<message::Message>()
            .add_systems(Update, packet_systems::client_recv_packet_system)
            .add_systems(Update, packet_systems::send_packet_system)
            .add_systems(Update, packet_systems::auto_heartbeat_system)
            .add_systems(Update, client_connection_handler)
            .add_systems(Update, NetworkTransform::sync_network_transforms)
            .add_systems(Update, auto_heartbeat_system)
            .add_systems(Update, sync_network_transforms)
            .add_systems(Update, listen_events)
            .insert_resource(PlayerId(0));
    }
}

fn client_connection_handler(mut events: EventReader<NetworkEvent>, mut messages: EventWriter<Message>) {
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

fn listen_game_events(
    mut commands: Commands,
    mut messages: EventReader<Message>,
    mut local_player_id: ResMut<PlayerId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut networked_entities: Query<(&NetworkObject, Entity)>,
    mut networked_objects: Query<(&NetworkObject, &mut NetworkTransform)>,
    _timer: Res<Time>,
) {
    for message in messages.iter() {
        println!("{:?}", message);
        match message {
            // TODO: Pass these functions into the ClientPlugin
            Spawn(id, pos, object_type, object_id) if (*id == *local_player_id) => {
                crate::game::client::spawn_network_object(object_type, *object_id, *id, *pos, &mut commands);
                *local_player_id = *id;
            }
            Spawn(id, pos, object_type, object_id) => crate::game::client::spawn_network_facade_object(
                object_type,
                *object_id,
                *id,
                *pos,
                &mut commands,
                &mut meshes,
                &mut materials,
            ),
            NetworkPosition(received_player_id, pos, _object_id) => {
                NetworkTransform::update_last_pos(received_player_id, pos, &mut networked_objects);
            }
            Despawn(_, object_id) => {
                for (object, entity) in networked_entities.iter_mut() {
                    if object.id == *object_id {
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
    networked_entities: Query<(&NetworkObject, Entity)>,
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