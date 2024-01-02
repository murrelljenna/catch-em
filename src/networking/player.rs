use bevy::ecs::system::Resource;

use rand::Rng;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(PartialEq, Debug, Serialize, Hash, Deserialize, Resource, Eq, Clone, Copy)]
pub struct PlayerId(pub u8);

#[derive(Resource, Default, Debug)]
pub struct Players {
    pub players: HashMap<PlayerId, SocketAddr>,
}

impl Players {
    pub fn for_all_except<F>(&self, excluded_id: PlayerId, mut action: F)
    where
        F: FnMut(&SocketAddr),
    {
        for (player_id, value) in &self.players {
            if *player_id != excluded_id {
                action(value);
            }
        }
    }

    pub fn add_player(&mut self, id: PlayerId, addr: SocketAddr) {
        self.players.insert(id, addr);
    }

    pub fn generate_id() -> PlayerId {
        let mut rng = rand::thread_rng();

        // Generate a random i16 value in the range [-32768, 32767]
        return PlayerId(rng.gen());
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.remove(&id);
    }

    pub fn player_from_socket(&mut self, addr: SocketAddr) -> Option<PlayerId> {
        for (key, value) in self.players.iter() {
            if *value == addr {
                return Some(key.clone());
            }
        }
        None
    }
}
