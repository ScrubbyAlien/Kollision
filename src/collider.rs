use bevy::math::ops::abs;
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

// todo: add system for pair pruning
// todo: add system for bounding box pruning
// todo: add system for quad tree pruning

#[derive(Component)]
pub struct CircleCollider {
    radius: f32,
    pub position: Vec3,
}

impl CircleCollider {
    pub fn new(radius: f32, transform: &Transform) -> Self {
        CircleCollider { radius, position: transform.translation }
    }

    pub fn collide_with_circle(&self, circle: &CircleCollider) -> bool {
        let collision_distance_sqr = self.radius + circle.radius;
        let collision_distance_sqr = collision_distance_sqr * collision_distance_sqr;

        let actual_distance_sqr = self.position.distance_squared(circle.position);

        actual_distance_sqr <= collision_distance_sqr
    }

    pub fn collide_with_capsule(&self, capsule: &CapsuleCollider) -> bool {
        // for simplicity, we assume the capsule is aligned to the x-axis

        let left_x = capsule.position.x - capsule.length / 2.;
        let right_x = left_x + capsule.length;
        let y = capsule.position.y;

        let mut distance: f32 = 0.;

        // circle is above rectangle portion of capsule
        if self.position.x > left_x && self.position.x < right_x {
            distance = abs(y - self.position.y);
            distance = distance * distance;
        }

        // circle is left of rectangle
        if self.position.x <= left_x {
            let difference: Vec2 = Vec2::new(self.position.x - left_x, self.position.y - y);
            distance = difference.length_squared();
        }

        // circle is right of rectangle
        if self.position.x >= right_x {
            let difference: Vec2 = Vec2::new(self.position.x - right_x, self.position.y - y);
            distance = difference.length_squared();
        }

        let collision_distance = capsule.radius + self.radius;
        distance <= collision_distance * collision_distance
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

    // pub fn collide_with_circle(&self, circle: &CircleCollider) -> bool {
    // }
    //
    // pub fn collide_with_capsule(&self, capsule: &CapsuleCollider) -> bool {
    // }
    //
    // pub fn bounding_box(&self) -> Rect {
    // }
    //
    // pub fn update_position(&mut self, new_position: Vec3) {
    //     self.position = new_position;
    // }
}