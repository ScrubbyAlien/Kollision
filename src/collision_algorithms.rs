use std::time::Instant;
use bevy::prelude::*;
use crate::ball::Ball;
use crate::capsule::Capsule;
use crate::collider::*;

#[derive(Message)]
pub struct CollisionMessage {
    pub entity1: Entity,
    pub entity2: Entity,
    pub info: CollisionInfo,
}

fn simple_capsule_collision(
    e1: Entity,
    circle: &CircleCollider,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) {
    for (e2, capsule, _) in capsules {
        if let Some(collided) = circle.collide_with_capsule(capsule) {
            collision_writer.write(CollisionMessage {
                entity1: e1,
                entity2: e2,
                info: collided,
            });
        }
    }
}

fn convert_indexable<'a>(
    balls: &'a Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>
) -> Vec<(Entity, &'a CircleCollider, &'a BoxCollider)>
{
    let mut balls_vec: Vec<(Entity, &CircleCollider, &BoxCollider)> = Vec::new();

    for tuple in balls {
        balls_vec.push(tuple);
    }

    balls_vec
}


pub fn no_algorithm(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    let start = Instant::now();

    for (e1, circle, _) in circles {
        simple_capsule_collision(e1, circle, capsules, collision_writer);

        'circles: for (e2, other_circle, _) in circles {
            if e1.eq(&e2) { continue 'circles; }
            if let Some(collided) = circle.collide_with_circle(other_circle) {
                collision_writer.write(CollisionMessage {
                    entity1: e1,
                    entity2: e2,
                    info: collided,
                });
            }
        }
    }

    start.elapsed().as_nanos()
}

pub fn pair_detection(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    let indexable = convert_indexable(circles); // convert to vec for slice 
    let start = Instant::now();

    for (index, (e1, circle, _)) in indexable.iter().enumerate() {
        simple_capsule_collision(*e1, circle, capsules, collision_writer);

        for (e2, other_circle, _) in indexable[index + 1..].iter() {
            if let Some(collided) = circle.collide_with_circle(other_circle) {
                collision_writer.write(CollisionMessage {
                    entity1: *e1,
                    entity2: *e2,
                    info: collided,
                });
            }
        }
    }

    start.elapsed().as_nanos()
}

fn bounded_capsule_collision(
    e1: Entity,
    circle: &CircleCollider,
    circle_bound: &BoxCollider,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) {
    for (e2, capsule, bound) in capsules {
        if !bounding_box_overlaps(circle_bound.absolute_rect, bound.absolute_rect) { continue; }
        if let Some(collided) = circle.collide_with_capsule(capsule) {
            collision_writer.write(CollisionMessage {
                entity1: e1,
                entity2: e2,
                info: collided,
            });
        }
    }
}

fn bounding_box_overlaps(rect1: Rect, rect2: Rect) -> bool {
    let left_or_above = rect1.max.x < rect2.min.x || rect1.min.y > rect2.max.y;
    let right_or_beneath = rect1.min.x > rect2.max.x || rect1.max.y < rect2.min.y;
    left_or_above || right_or_beneath
}

pub fn pair_bounding_box(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    let indexable = convert_indexable(circles); // convert to vec for slice
    let start = Instant::now();

    for (index, (e1, circle, bound)) in indexable.iter().enumerate() {
        bounded_capsule_collision(*e1, circle, bound, capsules, collision_writer);

        'circles: for (e2, other_circle, other_bound) in indexable[index + 1..].iter() {
            if !bounding_box_overlaps(bound.absolute_rect, other_bound.absolute_rect) {
                continue 'circles;
            }

            if let Some(collided) = circle.collide_with_circle(other_circle) {
                collision_writer.write(CollisionMessage {
                    entity1: *e1,
                    entity2: *e2,
                    info: collided,
                });
            }
        }
    }

    start.elapsed().as_nanos()
}

pub fn quad_tree(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    todo!()
}

