use crate::networking::components::{NetworkObject, NetworkTransform};
use crate::networking::message::serialize;
use crate::networking::message::Message::NetworkPosition;
use crate::networking::packet_systems::Socket;
use crate::networking::resources::PlayerId;
use crate::networking::Transport;
use bevy::prelude::{Entity, Query, Res, ResMut, Transform};


pub fn sync_network_transforms(
    socket: Res<Socket>,
    mut transport: ResMut<Transport>,
    mut query: Query<(&NetworkObject, Entity, &NetworkTransform, &mut Transform)>,
    player_id: Res<PlayerId>,
) {
    for (net_obj, _, _, transform) in query.iter_mut() {
        transport.send(
            socket
                .0
                .peer_addr()
                .expect("Socket address could not be found"),
            &serialize(NetworkPosition(
                *player_id,
                transform.translation,
                net_obj.id,
            )),
        )
    }
}
