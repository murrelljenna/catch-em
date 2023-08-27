use std;
use bevy::prelude::Vec3;
use bytes::Bytes;
use crate::networking::message::Message::SpawnPlayer;

#[derive(PartialEq, Debug)]
pub enum Message {
    SpawnPlayer(Vec3)
}

pub fn deserialize(bytes: Bytes) -> Option<Message> {
    if bytes.len() != 12 {
        return None
    }
    let byte_ptr = bytes.as_ptr();
    unsafe {
        let x = f32::from_le_bytes([
            *byte_ptr.offset(0),
            *byte_ptr.offset(1),
            *byte_ptr.offset(2),
            *byte_ptr.offset(3),
        ]);
        let y = f32::from_le_bytes([
            *byte_ptr.offset(4),
            *byte_ptr.offset(5),
            *byte_ptr.offset(6),
            *byte_ptr.offset(7),
        ]);
        let z = f32::from_le_bytes([
            *byte_ptr.offset(8),
            *byte_ptr.offset(9),
            *byte_ptr.offset(10),
            *byte_ptr.offset(11),
        ]);

        Some(SpawnPlayer(Vec3::new(x, y, z)))
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_extract_floats_from_bytes() {
        let byte_array: Vec<u8> = Vec::from([
            1.0f32.to_le_bytes()[0], 1.0f32.to_le_bytes()[1], 1.0f32.to_le_bytes()[2], 1.0f32.to_le_bytes()[3],   // Represents the float 1.0
            2.0f32.to_le_bytes()[0], 2.0f32.to_le_bytes()[1], 2.0f32.to_le_bytes()[2], 2.0f32.to_le_bytes()[3],   // Represents the float 2.0
            3.5f32.to_le_bytes()[0], 3.5f32.to_le_bytes()[1], 3.5f32.to_le_bytes()[2], 3.5f32.to_le_bytes()[3],   // Represents the float 3.5
        ]);

        let result = deserialize(Bytes::from(byte_array));

        assert_eq!(result, Some(SpawnPlayer(Vec3::new(1.0, 2.0, 3.5))));
    }

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