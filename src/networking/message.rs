use crate::networking::message::Message::Spawn;
use crate::networking::player::{NetworkObjectType, PlayerId};
use bevy::ecs::event::Event;
use bevy::prelude::Vec3;
use bytes::Bytes;
use serde_bytes::ByteBuf;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std;

#[derive(PartialEq, Debug, Serialize, Deserialize, Event, Copy, Clone)]
pub enum Message {
    Spawn(PlayerId, Vec3, NetworkObjectType, u8),
    PlayerPosition(PlayerId, Vec3, u8),
    NetworkInput { w: bool, s: bool, a: bool, d: bool },
    // Used in initial server->client handshake to pass network info to client
    ServerAcknowledgement(PlayerId),
    ClientAcknowledgement(PlayerId),
}

pub fn serialize(message: Message) -> Bytes {
    let x: Vec<u8> = serde_cbor::to_vec(&message).expect("Ahhh");
    Bytes::from_iter(x)
}

pub fn deserialize(bytes: Bytes) -> Message {
    serde_cbor::from_slice(&bytes).expect("Deserialization failed")
}
