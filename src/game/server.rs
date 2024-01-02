use std::{net::UdpSocket, time::Duration};

use crate::networking::handshake::server_handshake;

use crate::game::entities::DEFAULT_SPAWN_POINT;
use crate::networking::components::{NetworkObject, NetworkObjectType, NetworkObjects};
use crate::networking::message::{serialize, Message};
use crate::networking::packet_systems::Socket;
use crate::networking::player::Players;
use crate::networking::{NetworkEvent, ServerPlugin, Transport};
use bevy::log::Level;
use bevy::time::TimePlugin;
use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};

const LISTEN_ADDRESS: &str = "127.0.0.1:8080";

pub fn main() {
    let socket = UdpSocket::bind(LISTEN_ADDRESS).expect("could not bind socket");
    socket
        .set_nonblocking(true)
        .expect("could not set socket to be nonblocking");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("could not set read timeout");

    info!("Server now listening on {}", LISTEN_ADDRESS);

    App::new()
        // run the server at a reduced tick rate (100 ticks per minute)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
            1. / 30.,
        )))
        .insert_resource(Socket(socket))
        .insert_resource(Players::default())
        .insert_resource(NetworkObjects::default())
        .add_plugins(TimePlugin::default())
        .add_plugins(LogPlugin {
            filter: "".to_string(),
            level: Level::INFO,
        })
        .add_plugins(ServerPlugin)
        .add_systems(Update, connection_handler)
        .run();
}

fn connection_handler(
    mut events: EventReader<NetworkEvent>,
    mut transport: ResMut<Transport>,
    mut players: ResMut<Players>,
    mut network_objects: ResMut<NetworkObjects>,
) {
    for event in events.iter() {
        match event {
            NetworkEvent::Connected(handle) => {
                info!("{}: connected!", handle);
                server_handshake(handle, &mut transport);
            }
            NetworkEvent::Disconnected(handle) => {
                info!("{}: disconnected!", handle);
                let player_id = players
                    .player_from_socket(*handle)
                    .expect("Disconnected user not recorded in players");

                let player_objects = network_objects.objects_of_player(player_id);
                players.players.remove(&player_id);
                for object in player_objects {
                    for player_addr in players.players.values() {
                        network_objects.objects.remove(&object);
                        info!("{}: Sending despawn message", player_addr);
                        transport.send(
                            *player_addr,
                            &serialize(Message::Despawn(player_id, object.id)),
                        )
                    }
                }
            }
            NetworkEvent::RawMessage(handle, msg) => match msg {
                Message::PlayerPosition(player_id, pos, object_id) => {
                    players.for_all_except(*player_id, |addr| {
                        transport.send(
                            *addr,
                            &serialize(Message::PlayerPosition(*player_id, *pos, *object_id)),
                        );
                    });

                    let net_obj = network_objects
                        .objects
                        .keys()
                        .find(|net_obj| net_obj.id == *object_id)
                        .expect("Invalid id sent by client")
                        .clone();

                    network_objects.objects.insert(
                        NetworkObject {
                            id: net_obj.id,
                            owner: *player_id,
                            object_type: net_obj.object_type,
                        },
                        *pos,
                    );
                }
                Message::ClientAcknowledgement(player_id) => {
                    let obj_id = NetworkObject::generate_id();

                    let other_clients_message = Message::Spawn(
                        *player_id,
                        Vec3 {
                            x: 1f32,
                            y: 1f32,
                            z: 1f32,
                        },
                        NetworkObjectType::Player,
                        obj_id,
                    );

                    for player_addr in players.players.values() {
                        transport.send(*player_addr, &serialize(other_clients_message));
                    }

                    players.add_player(*player_id, *handle);

                    let message = Message::Spawn(
                        *player_id,
                        DEFAULT_SPAWN_POINT,
                        NetworkObjectType::Player,
                        obj_id,
                    );

                    for (network_obj, value) in network_objects.objects.iter() {
                        transport.send(
                            *handle,
                            &serialize(Message::Spawn(
                                network_obj.owner,
                                *value,
                                network_obj.object_type,
                                network_obj.id,
                            )),
                        )
                    }

                    network_objects.objects.insert(
                        NetworkObject {
                            id: obj_id,
                            owner: *player_id,
                            object_type: NetworkObjectType::Player,
                        },
                        Vec3 {
                            x: 1f32,
                            y: 1f32,
                            z: 1f32,
                        },
                    );

                    transport.send(*handle, &serialize(message));
                }
                _ => info!("{} sent a message: {:?}", handle, msg),
            },
            NetworkEvent::SendError(err, msg) => {
                error!(
                    "NetworkEvent::SendError (payload [{:?}]): {:?}",
                    msg.payload, err
                );
            }
            NetworkEvent::RecvError(err) => {
                error!("NetworkEvent::RecvError: {:?}", err);
            }
        }
    }
}
