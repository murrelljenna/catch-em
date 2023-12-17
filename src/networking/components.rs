use bevy::ecs::system::Resource;
use bevy::math::Vec3;
use bevy::prelude::Component;

use crate::networking::player::PlayerId;
use rand::Rng;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;


#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct NetworkObject {
    pub id: u8,
    pub owner: PlayerId,
    pub object_type: NetworkObjectType,
}

impl NetworkObject {
    pub fn generate_id() -> u8 {
        let mut rng = rand::thread_rng();

        // Generate a random i16 value in the range [-32768, 32767]
        return rng.gen();
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Copy, Clone, Hash, Eq)]
pub enum NetworkObjectType {
    Player,
}

#[derive(Resource, Default, Debug)]
pub struct NetworkObjects {
    pub objects: HashMap<NetworkObject, Vec3>,
}
