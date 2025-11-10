use bevy::prelude::*;
use crate::physics::RigidBody;

#[derive(Component)]
pub struct Capsule {
    radius: f32,
    length: f32,
}

pub fn create_capsule(
    length: f32,
    radius: f32,
    color: Color,
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Capsule, Mesh2d, MeshMaterial2d<ColorMaterial>, Transform)
{
    let capsule = Capsule { radius, length };
    let mesh = meshes.add(Capsule2d::new(radius, length));
    let material = materials.add(color);

    (
        capsule,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        transform
    )
}