/*
    This controls how the server and client decide initial network details once
    the server receives the initial connection. These details are:
        - PlayerId for the newly connected client

    The client cannot receive any other server communication until this handshake
    is completed.
 */

use std::net::SocketAddr;
use bevy::prelude::{EventReader, Local, Res, ResMut};
use bevy::ecs::system::Resource;
use crate::networking::message::{Message, serialize};
use crate::networking::message::Message::{ClientAcknowledgement, ServerAcknowledgement};
use crate::networking::player::{PlayerId, Players};
use crate::networking::systems::Socket;
use crate::networking::Transport;
use serde::Serialize;

#[derive(Resource, Debug)]
pub enum ConnectionStatus {
    Initial, // Client has just sent connection to server
    Acknowledged, // Client has received server acknowledgement
    Complete // Client has sent server acknowledgement
}

pub fn listen_handshake_events(mut messages: EventReader<Message>, socket: Res<Socket>, mut transport: ResMut<Transport>, mut local_player_id: ResMut<PlayerId>, mut connection_status: ResMut<ConnectionStatus>) {
    for message in messages.iter() {
        match message {
            ServerAcknowledgement(id) => client_handshake(id, &socket, &mut transport, &mut local_player_id, &mut connection_status),
            _ => ()
        }
    };
}

pub fn server_handshake(handle: &SocketAddr, transport: &mut ResMut<Transport>) {
    // Generate player id for client
    let player_id: PlayerId = Players::generate_id();
    // Send client this id
    let message = Message::ServerAcknowledgement(player_id);

    transport.send(*handle, &serialize(message));
}

fn client_handshake(assigned_player_id: &PlayerId, socket: &Res<Socket>, mut transport: &mut ResMut<Transport>, mut local_player_id: &mut ResMut<PlayerId>, mut connection_status: &mut ResMut<ConnectionStatus>) {
    **local_player_id = *assigned_player_id;
    **connection_status = ConnectionStatus::Complete;
    println!("Doing client handshake");
    let message = ClientAcknowledgement(*assigned_player_id);

    transport.send(socket.0.peer_addr().expect("Socket address could not be found"), &serialize(message));
}

