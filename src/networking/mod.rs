mod events;
mod raw_message;
mod systems;
mod transport;
pub mod server;
pub mod client;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

pub use self::events::NetworkEvent;
pub use self::transport::Transport;

use bevy::prelude::*;

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

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkResource::default())
            .insert_resource(transport::Transport::new())
            .add_event::<events::NetworkEvent>()
            .add_systems(Update,systems::server_recv_packet_system)
            .add_systems(Update,systems::send_packet_system)
            .add_systems(Update, systems::idle_timeout_system);
    }
}

#[derive(Resource)]
pub struct HeartbeatTimer(Timer);

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(transport::Transport::new())
            .insert_resource(HeartbeatTimer(Timer::from_seconds(DEFAULT_HEARTBEAT_TICK_RATE_SECS, Default::default())))
            .add_event::<events::NetworkEvent>()
            .add_systems(Update, systems::client_recv_packet_system)
            .add_systems(Update, systems::send_packet_system)
            .add_systems(Update, systems::auto_heartbeat_system);
    }
}
