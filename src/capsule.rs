use std::f32::consts::PI;
use bevy::prelude::*;
use crate::collider::{BoxCollider, CapsuleCollider};

#[derive(Component)]
pub struct Capsule {
    radius: f32,
    length: f32,
}

#[derive(Bundle)]
pub struct CapsuleBundle {
    capsule: Capsule,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    bounds: BoxCollider,
    collider: CapsuleCollider,
    transform: Transform,
}

pub fn create_capsule(
    length: f32,
    radius: f32,
    color: Color,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> CapsuleBundle
{
    let capsule = Capsule { radius, length };
    let mesh = meshes.add(Capsule2d::new(radius, length));
    let material = materials.add(color);
    let mut transform = Transform::from_translation(position);
    transform.rotate_axis(Dir3::Z, PI / 2.);
    let collider = CapsuleCollider::new(radius, length, &transform);

    CapsuleBundle {
        capsule,
        mesh: Mesh2d(mesh),
        material: MeshMaterial2d(material),
        bounds: BoxCollider::new(collider.relative_bound(), &transform),
        collider,
        transform,
    }
}