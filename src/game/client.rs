use crate::networking::components::NetworkObjectType;

use crate::networking::handshake::{ConnectionStatus};

use crate::networking::resources::PlayerId;

use crate::game::entities::{spawn_player, spawn_player_facade};

use crate::networking::{ClientPlugin};
use crate::{display_text, manage_cursor, respawn, scene_colliders, setup};
use bevy::prelude::*;
use bevy_fps_controller::controller::FpsControllerPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::RapierConfiguration;

pub fn main(socket_addr: String) {
    App::new()
        .insert_resource(ConnectionStatus::Initial)
        .add_plugins(ClientPlugin("127.0.0.1:8080".parse().unwrap(), socket_addr))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (manage_cursor, scene_colliders, display_text, respawn),
        )
        .run();
}

pub(crate) fn spawn_network_object(
    object_type: &NetworkObjectType,
    object_id: u8,
    id: PlayerId,
    pos: Vec3,
    mut commands: &mut Commands,
) {
    println!("Spawning net object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player(id, object_id, pos, &mut commands);
        }
    }
}

pub(crate) fn spawn_network_facade_object(
    object_type: &NetworkObjectType,
    object_id: u8,
    id: PlayerId,
    pos: Vec3,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    println!("Spawning facade object");
    match object_type {
        NetworkObjectType::Player => {
            spawn_player_facade(
                id,
                object_id,
                pos,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    }
}
