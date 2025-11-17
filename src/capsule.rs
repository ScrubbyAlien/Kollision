use std::f32::consts::PI;
use bevy::prelude::*;
use crate::collider::CapsuleCollider;

#[derive(Component)]
pub struct Capsule {
    radius: f32,
    length: f32,
}

pub fn create_capsule(
    length: f32,
    radius: f32,
    color: Color,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Capsule, Mesh2d, MeshMaterial2d<ColorMaterial>, CapsuleCollider, Transform)
{
    let capsule = Capsule { radius, length };
    let mesh = meshes.add(Capsule2d::new(radius, length));
    let material = materials.add(color);
    let mut transform = Transform::from_translation(position);
    transform.rotate_axis(Dir3::Z, PI / 2.);

    (
        capsule,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        CapsuleCollider::new(radius, length, &transform),
        transform
    )
}