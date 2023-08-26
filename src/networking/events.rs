use std;
use std::{io, net::SocketAddr};

use bytes::Bytes;
use bevy::ecs::event::Event;

use super::message::Message;

#[derive(Event)]
pub enum NetworkEvent {
    // A message was received from a client
    Message(SocketAddr, Bytes),
    // A new client has connected to us
    Connected(SocketAddr),
    // A client has disconnected from us
    Disconnected(SocketAddr),
    // An error occurred while receiving a message
    RecvError(io::Error),
    // An error occurred while sending a message
    SendError(io::Error, Message),
}
