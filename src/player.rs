use std::f32::consts::TAU;

use bevy::{
    prelude::*,
};
use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;
use crate::networking::player::{NetworkObject, PlayerId, Players};

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub(crate) fn spawn_player_facade(id: PlayerId, object_id: u8, pos: Vec3, commands: &mut Commands, mut meshes: &mut ResMut<Assets<Mesh>>, mut materials: &mut ResMut<Assets<StandardMaterial>>,) {
    commands.spawn((
        Collider::capsule(pos, pos * 1.5, 0.5),
        NetworkObject { id: object_id, owner: id },
        LockedAxes::ROTATION_LOCKED,
        ActiveEvents::COLLISION_EVENTS,
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5,
                rings: 0,
                depth: 4.0,
                latitudes: 20,
                longitudes: 20,
                uv_profile: Default::default(),
            })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(SPAWN_POINT),
            ..default()
        }
        )
    );
}

#[derive(Bundle)]
struct FPSControllerBundle {
    input: FpsControllerInput,
    controller: FpsController
}

pub(crate) fn spawn_player(id: PlayerId, object_id: u8, pos: Vec3, commands: &mut Commands) {
    commands.spawn((
        Collider::capsule(pos, pos * 1.5, 0.5),
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true }, // Prevent clipping when going fast
        TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
        LogicalPlayer(0),
        FPSControllerBundle{
            input: FpsControllerInput {
            pitch: -TAU / 12.0,
            yaw: TAU * 5.0 / 8.0,
            ..default()
        },
        controller: FpsController {
            air_acceleration: 80.0,
            ..default()
        }},
        (NetworkObject { id: object_id, owner: id })
    ));
}