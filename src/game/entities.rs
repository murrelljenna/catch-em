use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::networking::components::{NetworkObject, NetworkObjectType, NetworkTransform};
use crate::networking::resources::PlayerId;
use bevy_fps_controller::controller::*;

pub const DEFAULT_SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

pub fn spawn_player_facade(
    id: PlayerId,
    object_id: u8,
    pos: Vec3,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Collider::capsule(pos, pos * 1.2, 0.5),
        NetworkObject {
            id: object_id,
            owner: id,
            object_type: NetworkObjectType::Player,
            is_owned: false
        },
        NetworkTransform {
            last_pos: Transform::from_translation(pos).translation,
        },
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
            transform: Transform::from_translation(pos),
            ..default()
        },
    ));
}

#[derive(Bundle)]
struct FPSControllerBundle {
    input: FpsControllerInput,
    controller: FpsController,
}

#[derive(Bundle)]
struct NetworkTransformBundle {
    transform: NetworkTransform,
    object: NetworkObject
}

impl NetworkTransformBundle {
    pub fn new(object_id: u8, owner_id: PlayerId, object_type: NetworkObjectType, start_pos: Vec3, is_owned: bool) -> NetworkTransformBundle {
        return NetworkTransformBundle {
            transform: NetworkTransform {
                last_pos: Transform::from_translation(start_pos).translation,
            },
            object: NetworkObject {
                id: object_id,
                owner: owner_id,
                object_type,
                is_owned
            }
        }
    }
}

pub fn spawn_player(id: PlayerId, object_id: u8, pos: Vec3, commands: &mut Commands) {
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
        NetworkObject {
            id: object_id,
            owner: id,
            object_type: NetworkObjectType::Player,
            is_owned: true
        },
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true }, // Prevent clipping when going fast
        TransformBundle::from_transform(Transform::from_translation(pos)),
        LogicalPlayer(0),
        FPSControllerBundle {
            input: FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            controller: FpsController {
                air_acceleration: 80.0,
                ..default()
            },
        }
    ));
}
