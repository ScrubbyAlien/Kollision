use bevy::math::NormedVectorSpace;
use bevy::math::ops::{abs, sqrt};
use bevy::prelude::*;

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_collider_positions);
    }
}

fn update_collider_positions(query: Query<(&mut CircleCollider, &Transform)>) {
    for (mut circle_collider, &transform) in query {
        circle_collider.update_position(transform.translation);
    }
}

pub struct CollisionInfo {
    pub normal: Vec2,
    pub overlap: f32,
}

impl CollisionInfo {
    fn new(normal: Vec2, overlap: f32) -> Self {
        Self { normal, overlap }
    }

    pub fn offset(&self) -> Vec2 {
        self.normal * self.overlap
    }
}

#[derive(Component)]
pub struct CircleCollider {
    radius: f32,
    pub position: Vec3,
}

impl CircleCollider {
    pub fn new(radius: f32, transform: &Transform) -> Self {
        CircleCollider { radius, position: transform.translation }
    }

    pub fn collide_with_circle(&self, circle: &CircleCollider) -> Option<CollisionInfo> {
        let collision_distance_sqr = self.radius + circle.radius;
        let collision_distance_sqr = collision_distance_sqr * collision_distance_sqr;

        let actual_distance_sqr = self.position.distance_squared(circle.position);

        if actual_distance_sqr <= collision_distance_sqr {
            let normal = Vec3::truncate(self.position - circle.position).normalize();
            let overlap = sqrt(collision_distance_sqr) - self.position.distance(circle.position);
            Some(CollisionInfo::new(normal, overlap))
        } else {
            None
        }
    }

    pub fn collide_with_capsule(&self, capsule: &CapsuleCollider) -> Option<CollisionInfo> {
        // for simplicity, we assume the capsule is aligned to the x-axis

        let left_x = capsule.position.x - capsule.length / 2.;
        let right_x = left_x + capsule.length;
        let y = capsule.position.y;

        let mut distance_sqr = 0.;
        let mut normal = Vec2::new(0., 0.);
        let mut overlap = 0.;

        let col_distance = capsule.radius + self.radius;

        // circle is above rectangle portion of capsule
        if self.position.x > left_x && self.position.x < right_x {
            distance_sqr = abs(y - self.position.y);
            distance_sqr = distance_sqr * distance_sqr;
            // assuming x-aligned capsule
            normal = Vec2::new(0., self.position.y - capsule.position.y).normalize();
            overlap = col_distance - sqrt(distance_sqr);
        }

        // circle is left of rectangle
        if self.position.x <= left_x {
            let difference: Vec2 = Vec2::new(self.position.x - left_x, self.position.y - y);
            distance_sqr = difference.length_squared();
            normal = (Vec3::truncate(self.position) - Vec2::new(left_x, y)).normalize();
            overlap = col_distance - sqrt(distance_sqr);
        }

        // circle is right of rectangle
        if self.position.x >= right_x {
            let difference: Vec2 = Vec2::new(self.position.x - right_x, self.position.y - y);
            distance_sqr = difference.length_squared();
            normal = (Vec3::truncate(self.position) - Vec2::new(right_x, y)).normalize();
            overlap = col_distance - sqrt(distance_sqr);
        }

        let collision_distance = capsule.radius + self.radius;
        if distance_sqr <= collision_distance * collision_distance {
            Some(CollisionInfo { normal, overlap })
        } else {
            None
        }
    }


    pub fn bounding_box(&self) -> Rect {
        todo!()
    }

    pub fn update_position(&mut self, new_position: Vec3) {
        self.position = new_position;
    }
}


#[derive(Component)]
pub struct CapsuleCollider {
    length: f32,
    radius: f32,
    pub position: Vec3,
}

impl CapsuleCollider {
    pub fn new(radius: f32, length: f32, transform: &Transform) -> Self {
        CapsuleCollider { length, radius, position: transform.translation }
    }
}