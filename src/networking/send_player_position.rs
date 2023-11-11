use bevy::prelude::{Entity, Query, Res, ResMut, Transform, Vec3};
use bevy_fps_controller::controller::{FpsController, FpsControllerInput};
use crate::networking::message::Message::PlayerPosition;
use crate::networking::message::serialize;
use crate::networking::player::{PlayerId, Players};
use crate::networking::systems::Socket;
use crate::networking::Transport;

pub fn send_player_position(
    socket: Res<Socket>, mut transport: ResMut<Transport>, mut query: Query<(
        Entity,
        &mut FpsController,
        &mut Transform,
    )>,
    player_id: Res<PlayerId>
) {
    for (_, _, mut transform) in query.iter_mut() {
        transport.send(socket.0.peer_addr().expect("Socket address could not be found"), &serialize(PlayerPosition(*player_id, transform.translation)))
    }
}

pub fn broadcast_player_position(
    player_id: Res<PlayerId>,
    players: Res<Players>,
    mut transport: ResMut<Transport>,
    pos: Vec3
) {
    for (_, socket_addr) in players.players.iter() {
        transport.send(*socket_addr, &serialize(PlayerPosition(*player_id, pos)))
    }
}