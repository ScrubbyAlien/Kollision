use bevy::prelude::*;
use bevy_trait_query::queryable;

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

// todo: add system for pair pruning
// todo: add system for bounding box pruning
// todo: add system for quad tree pruning


#[queryable]
pub trait Collider {
    fn collide_with_circle(&self, circle: CircleCollider) -> bool;
    fn collide_with_capsule(&self, capsule: CapsuleCollider) -> bool;
    fn bounding_box(&self) -> Rect;
}

#[derive(Component)]
pub struct CircleCollider {
    radius: f32,
    position: Vec3,
}

impl CircleCollider {
    fn new(radius: f32, position: Vec3) -> Self {
        CircleCollider { radius, position }
    }

    fn update_position(&mut self, new_position: Vec3) {
        self.position = new_position;
    }
}

impl Collider for CircleCollider {
    fn collide_with_circle(&self, circle: CircleCollider) -> bool {
        let collision_distance_sqr = self.radius + circle.radius;
        let collision_distance_sqr = collision_distance_sqr * collision_distance_sqr;

        let actual_distance_sqr = self.position.distance_squared(circle.position);

        actual_distance_sqr <= collision_distance_sqr
    }

    fn collide_with_capsule(&self, capsule: CapsuleCollider) -> bool {
        todo!()
    }

    fn bounding_box(&self) -> Rect {
        todo!()
    }
}


#[derive(Component)]
pub struct CapsuleCollider {
    length: f32,
    radius: f32,
    position: Vec3,
}

impl CapsuleCollider {
    fn new(radius: f32, length: f32, position: Vec3) -> Self {
        CapsuleCollider { length, radius, position }
    }

    fn update_position(&mut self, new_position: Vec3) {
        self.position = new_position;
    }
}

impl Collider for CapsuleCollider {
    fn collide_with_circle(&self, circle: CircleCollider) -> bool {
        todo!()
    }

    fn collide_with_capsule(&self, capsule: CapsuleCollider) -> bool {
        todo!()
    }

    fn bounding_box(&self) -> Rect {
        todo!()
    }
}