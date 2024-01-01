use crate::networking::components::NetworkObject;
use crate::networking::message::serialize;
use crate::networking::message::Message::PlayerPosition;
use crate::networking::packet_systems::Socket;
use crate::networking::resources::PlayerId;
use crate::networking::Transport;
use bevy::prelude::{Entity, Query, Res, ResMut, Transform};
use bevy_fps_controller::controller::FpsController;

pub fn send_player_position(
    socket: Res<Socket>,
    mut transport: ResMut<Transport>,
    mut query: Query<(&NetworkObject, Entity, &mut FpsController, &mut Transform)>,
    player_id: Res<PlayerId>,
) {
    for (net_obj, _, _, transform) in query.iter_mut() {
        transport.send(
            socket
                .0
                .peer_addr()
                .expect("Socket address could not be found"),
            &serialize(PlayerPosition(
                *player_id,
                transform.translation,
                net_obj.id,
            )),
        )
    }
}
