use bevy::ecs::system::Resource;

pub type PlayerId = i16;

#[derive(Resource, Default)]
pub struct Players {
    pub players: Vec<PlayerId>,
    pub lastPlayerId: PlayerId
}