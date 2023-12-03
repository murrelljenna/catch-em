use std;
use bevy::prelude::Vec3;
use bytes::Bytes;
use serde_bytes::ByteBuf;
use serde_derive::Serialize;
use serde_derive::Deserialize;
use bevy::ecs::event::Event;
use crate::networking::message::Message::SpawnOwned;
use crate::networking::message::Message::SpawnNetworked;
use crate::networking::player::{NetworkObjectType, PlayerId};

#[derive(PartialEq, Debug, Serialize, Deserialize, Event, Copy, Clone)]
pub enum Message {
    SpawnNetworked(PlayerId, Vec3, NetworkObjectType),
    SpawnOwned(PlayerId, Vec3, NetworkObjectType),
    PlayerPosition(PlayerId, Vec3),
    NetworkInput {
        w: bool,
        s: bool,
        a: bool,
        d: bool
    }
}

pub fn serialize(message: Message) -> Bytes {
    let x: Vec<u8> = serde_cbor::to_vec(&message).expect("Ahhh");
    Bytes::from_iter(x)
}

pub fn deserialize(bytes: Bytes) -> Message {
    serde_cbor::from_slice(&bytes).expect("Deserialization failed")
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    /*#[test]
    fn test_extract_floats_from_bytes_insufficient_bytes() {
        let byte_array: [u8; 8] = [
            63, 128, 0, 0,     // Represents the float 1.0
            64, 0, 0, 0,       // Represents the float 2.0
        ];

        let byte_ptr: *const u8 = &byte_array as *const _;

        let result = deserialize(Bytes::from(byte_ptr));

        assert_eq!(result, None);
    }*/
}