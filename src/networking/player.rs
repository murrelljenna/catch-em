use core::fmt;
use std::collections::HashMap;
use std::net::SocketAddr;
use bevy::ecs::system::Resource;
use bevy::prelude::Component;
use rand::Rng;
use serde_derive::Serialize;
use serde_derive::Deserialize;

#[derive(PartialEq, Debug, Serialize, Hash, Deserialize, Resource, Eq, Clone, Copy)]
pub struct PlayerId(pub u8);

#[derive(Component)]
pub struct NetworkObject {
    pub player_id: PlayerId
}

#[derive(Resource, Default, Debug)]
pub struct Players {
    pub players: HashMap<PlayerId, SocketAddr>
}

impl Players {
    pub fn except_id(self, id: PlayerId) -> Players {
        let mut copied_map = HashMap::new();
        for (key, value) in &self.players {
            if *key != id {
                copied_map.insert(key.clone(), *value);
            }
        }

        return Players {players: copied_map };
    }

    pub fn except_addr(self, socket_addr: SocketAddr) -> Players {
        let mut copied_map = HashMap::new();
        for (key, value) in &self.players {
            if *value != socket_addr {
                copied_map.insert(key.clone(), *value);
            }
        }

        return Players {players: copied_map };
    }

    pub fn add_player(&mut self, id: PlayerId, addr: SocketAddr) {
        self.players.insert(id, addr);
    }

    pub fn generate_id() -> PlayerId {
        let mut rng = rand::thread_rng();

        // Generate a random i16 value in the range [-32768, 32767]
        return PlayerId(rng.gen());
    }
}

