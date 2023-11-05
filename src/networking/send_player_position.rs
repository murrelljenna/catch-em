use bevy::prelude::{Entity, Query, Res, ResMut, Transform};
use bevy_fps_controller::controller::{FpsController, FpsControllerInput};
use crate::networking::message::Message::PlayerPosition;
use crate::networking::message::serialize;
use crate::networking::systems::Socket;
use crate::networking::Transport;

pub fn send_player_position(
    socket: Res<Socket>, mut transport: ResMut<Transport>, mut query: Query<(
        Entity,
        &mut FpsController,
        &mut Transform,
    )>
) {
    for (_, _, mut transform) in query.iter_mut() {
        transport.send(socket.0.peer_addr().expect("Socket address could not be found"), &serialize(PlayerPosition(transform.translation)))
    }
}