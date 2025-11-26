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
    capsules: &Query<(Entity, &CapsuleCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) {
    for (e2, capsule) in capsules {
        if let Some(collided) = circle.collide_with_capsule(capsule) {
            collision_writer.write(CollisionMessage {
                entity1: e1,
                entity2: e2,
                info: collided,
            });
        }
    }
}

fn convert_indexable<'a>(balls: &'a Query<(Entity, &CircleCollider), With<Ball>>) -> Vec<(Entity, &'a CircleCollider)> {
    let mut balls_vec: Vec<(Entity, &CircleCollider)> = Vec::new();

    for tuple in balls {
        balls_vec.push(tuple);
    }

    balls_vec
}


pub fn no_algorithm(
    circles: &Query<(Entity, &CircleCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    let start = Instant::now();

    for (e1, circle) in circles {
        simple_capsule_collision(e1, circle, capsules, collision_writer);

        'circles: for (e2, other_circle) in circles {
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
    circles: &Query<(Entity, &CircleCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    let indexable = convert_indexable(circles); // convert to vec for slice 
    let start = Instant::now();

    for (index, (e1, circle)) in indexable.iter().enumerate() {
        simple_capsule_collision(*e1, circle, capsules, collision_writer);

        for (e2, other_circle) in indexable[index + 1..].iter() {
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

pub fn bounding_box(
    circles: &Query<(Entity, &CircleCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    todo!()
}


pub fn quad_tree(
    circles: &Query<(Entity, &CircleCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
) -> u128 {
    todo!()
}

