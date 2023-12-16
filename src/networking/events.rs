use std;
use std::{io, net::SocketAddr};

use crate::networking::message::Message;
use bevy::ecs::event::Event;
use bytes::Bytes;

use super::raw_message::RawMessage;

#[derive(Event)]
pub enum NetworkEvent {
    // A message was received from a client
    RawMessage(SocketAddr, Message),
    // A new client has connected to us
    Connected(SocketAddr),
    // A client has disconnected from us
    Disconnected(SocketAddr),
    // An error occurred while receiving a message
    RecvError(io::Error),
    // An error occurred while sending a message
    SendError(io::Error, RawMessage),
}
