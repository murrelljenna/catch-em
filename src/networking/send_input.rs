use crate::networking::message::serialize;
use crate::networking::message::Message::NetworkInput;
use crate::networking::systems::Socket;
use crate::networking::{NetworkEvent, Transport};
use bevy::input::Input;
use bevy::prelude::{KeyCode, Res, ResMut};

pub fn send_player_input(
    socket: Res<Socket>,
    keys: Res<Input<KeyCode>>,
    mut transport: ResMut<Transport>,
) {
    transport.send(
        socket
            .0
            .peer_addr()
            .expect("Socket address could not be found"),
        &serialize(NetworkInput {
            w: keys.pressed(KeyCode::W),
            s: keys.pressed(KeyCode::S),
            a: keys.pressed(KeyCode::A),
            d: keys.pressed(KeyCode::D),
        }),
    )
}
