use crate::networking::resources::PlayerId;
use bevy::ecs::system::Resource;
use bevy::math::Vec3;
use bevy::prelude::{Component, Query, Res, Time, Transform};

use rand::Rng;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct NetworkObject {
    pub id: u8,
    pub owner: PlayerId,
    pub object_type: NetworkObjectType,
    pub is_owned: bool
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

impl NetworkObjects {
    pub fn objects_of_player(&mut self, id: PlayerId) -> Vec<NetworkObject> {
        let mut net_objs = Vec::new();

        for object in self.objects.keys() {
            if object.owner == id {
                net_objs.push(object.clone());
            }
        }

        net_objs
    }
}

/*
Synchronizes the position of a NetworkObject across the network.
*/

#[derive(Component, Debug)]
pub struct NetworkTransform {
    pub last_pos: Vec3,
}

impl NetworkTransform {
    /*
    Grab the identified NetworkTransform and update its last_pos. This is generally performed as positions are received
    from the server.
     */
    pub fn update_last_pos(
        received_player_id: &PlayerId,
        received_position: &Vec3,
        networked_objects: &mut Query<(&NetworkObject, &mut NetworkTransform)>,
    ) {
        for (networked_object, mut transform) in networked_objects.iter_mut() {
            if networked_object.owner == *received_player_id {
                transform.last_pos = *received_position;
            }
        }
    }
    /*
    Iterate over all non-owned network transforms and lerp their positions smoothly.
     */
    pub fn sync_network_transforms(
        mut networked_objects: Query<(&mut Transform, &NetworkTransform)>,

        timer: Res<Time>,
    ) {
        for (mut transform, network_transform) in networked_objects.iter_mut() {
            println!("{:?}", network_transform.last_pos);
            if transform.translation == network_transform.last_pos {
                continue;
            }
            let incremental_adjust = 10f32 * timer.delta_seconds();
            let old_translation = transform.translation;
            transform.translation =
                old_translation.lerp(network_transform.last_pos, incremental_adjust);
        }
    }
}
