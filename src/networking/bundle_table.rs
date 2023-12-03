use std::collections::HashMap;
use bevy::prelude::{Assets, Bundle, Commands, Mesh, ResMut, StandardMaterial, Vec3};
use crate::player::player_bundle;

const BUNDLE_PAIRS: [(&str, Box<fn(pos: Vec3, commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) -> Box<dyn Bundle>>); 1] = [
    ("key1", player_bundle)
];

lazy_static::lazy_static! {
    static ref BUNDLE_TABLE: HashMap<&'static String, &'static dyn Bundle> = {
        let mut table = HashMap::new();
        table.insert()
    };
}
