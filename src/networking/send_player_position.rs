use bevy::prelude::{Entity, Query, Res, ResMut, Transform, Vec3};
use bevy_fps_controller::controller::{FpsController, FpsControllerInput};
use crate::networking::message::Message::PlayerPosition;
use crate::networking::message::serialize;
use crate::networking::player::{NetworkObject, PlayerId, Players};
use crate::networking::systems::Socket;
use crate::networking::Transport;

pub fn send_player_position(
    socket: Res<Socket>, mut transport: ResMut<Transport>, mut query: Query<(
        &NetworkObject,
        Entity,
        &mut FpsController,
        &mut Transform,
    )>,
    player_id: Res<PlayerId>
) {
    for (net_obj, _, _, mut transform) in query.iter_mut() {
        transport.send(socket.0.peer_addr().expect("Socket address could not be found"), &serialize(PlayerPosition(*player_id, transform.translation, net_obj.id)))
    }
}