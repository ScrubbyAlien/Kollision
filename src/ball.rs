use bevy::prelude::*;
use crate::collider::CircleCollider;
use crate::physics::RigidBody;

// #[cfg(not(target_arch = "wasm32"))]
// use bevy::sprite_render::{Wireframe2dConfig, Wireframe2dPlugin};

#[derive(Component)]
pub struct Ball {
    radius: f32,
}

pub fn create_ball(
    radius: f32,
    color: Color,
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Ball, Mesh2d, MeshMaterial2d<ColorMaterial>, RigidBody, CircleCollider, Transform)
{
    let ball = Ball { radius };
    let mesh = meshes.add(Circle::new(ball.radius));
    let material = materials.add(color);

    (
        ball,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        RigidBody::new().mass(radius * radius),
        CircleCollider::new(radius, &transform),
        transform
    )
}



