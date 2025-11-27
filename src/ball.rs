use bevy::prelude::*;
use crate::collider::{BoxCollider, CircleCollider};
use crate::physics::RigidBody;

// #[cfg(not(target_arch = "wasm32"))]
// use bevy::sprite_render::{Wireframe2dConfig, Wireframe2dPlugin};

#[derive(Component)]
pub struct Ball {
    radius: f32,
}

#[derive(Bundle)]
pub struct BallBundle {
    ball: Ball,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    body: RigidBody,
    bounds: BoxCollider,
    collider: CircleCollider,
    transform: Transform,
}

pub fn create_ball(
    radius: f32,
    color: Color,
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> BallBundle
{
    let ball = Ball { radius };
    let mesh = meshes.add(Circle::new(ball.radius));
    let material = materials.add(color);
    let collider = CircleCollider::new(radius, &transform);

    BallBundle {
        ball,
        mesh: Mesh2d(mesh),
        material: MeshMaterial2d(material),
        body: RigidBody::new().mass(radius * radius).gravity_scale(3.),
        bounds: BoxCollider::new(collider.relative_bound(), &transform),
        collider,
        transform,
    }
}



