use std::time::Instant;
use bevy::color::palettes::basic::{GREEN};
use bevy::prelude::*;
use crate::ball::Ball;
use crate::capsule::Capsule;
use crate::collider::*;
use crate::profiler::Profiler;

#[derive(Message)]
#[allow(unused)]
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
    !rect1.intersect(rect2).is_empty()
}

pub fn pair_bounding_box(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
    mut gizmos: Gizmos,
    debug: bool,
) -> u128 {
    let indexable = convert_indexable(circles); // convert to vec for slice
    let start = Instant::now();

    if debug {
        for (_, _, bound) in capsules {
            draw_gizmos_rect(&mut gizmos, &bound.absolute_rect);
        }
    }

    for (index, (e1, circle, bound)) in indexable.iter().enumerate() {
        if debug { draw_gizmos_rect(&mut gizmos, &bound.absolute_rect); }

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

#[allow(clippy::too_many_arguments)]
pub fn quad_tree(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    collision_writer: &mut MessageWriter<CollisionMessage>,
    window_rect: Rect,
    profiler: &mut ResMut<Profiler>,
    table_index: usize,
    sample_size_index: usize,
    mut gizmos: Gizmos,
    debug: bool,
) -> u128 {
    let start = Instant::now();

    let mut tree_entities: Vec<(Entity, Rect)> = Vec::new();
    for (e, _, b) in circles {
        tree_entities.push((e, b.absolute_rect));
    }
    for (e, _, b) in capsules {
        tree_entities.push((e, b.absolute_rect));
    }

    let quad_tree = QuadTree::new(window_rect, tree_entities);
    let flat_quad_tree = quad_tree.root_node.flatten_tree();


    let build_time = start.elapsed().as_nanos();
    profiler.record_cell_data_by_table_row_col_index(
        table_index,
        0,
        sample_size_index,
        build_time,
    );

    let traversal_start = Instant::now();


    // let mut number_of_checks = 0;
    // println!("{}", "new traversal");
    for node in flat_quad_tree {
        if debug { draw_gizmos_rect(&mut gizmos, &node.rect); }

        let mut non_checked_entities = node.contained_entities.clone();

        while !non_checked_entities.is_empty() {
            let e1 = non_checked_entities.remove(non_checked_entities.len() - 1);
            let mut candidates = non_checked_entities.clone();
            candidates.append(&mut node.get_overlapping_sub_tress(e1));

            for e2 in candidates {
                // number_of_checks += 1;
                if let Some(info) = evaluate_collision(circles, capsules, e1.0, e2.0) {
                    collision_writer.write(CollisionMessage {
                        entity1: e1.0,
                        entity2: e2.0,
                        info,
                    });
                }
            }
        }
    }

    // println!("{}", number_of_checks as f32 / (num_entities * num_entities / 2.));

    let traversal_time = traversal_start.elapsed().as_nanos();
    profiler.record_cell_data_by_table_row_col_index(
        table_index,
        1,
        sample_size_index,
        traversal_time,
    );

    start.elapsed().as_nanos()
}

fn draw_gizmos_rect(gizmos: &mut Gizmos, rect: &Rect) {
    gizmos.rect_2d(Isometry2d::new(rect.center(), Rot2::IDENTITY), rect.size(), GREEN);
}

fn evaluate_collision(
    circles: &Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: &Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    entity1: Entity,
    entity2: Entity,
) -> Option<CollisionInfo> {
    let e1_circle_col = circles.get(entity1);
    let e1_capsule_col = capsules.get(entity1);
    let e2_circle_col = circles.get(entity2);
    let e2_capsule_col = capsules.get(entity2);

    if let Ok(col1) = e1_circle_col && let Ok(col2) = e2_circle_col {
        col1.1.collide_with_circle(col2.1)
    } else if let Ok(col1) = e1_circle_col && let Ok(col2) = e2_capsule_col {
        col1.1.collide_with_capsule(col2.1)
    } else if let Ok(col1) = e1_capsule_col && let Ok(col2) = e2_circle_col {
        col2.1.collide_with_capsule(col1.1)
    } else { None }
}


/// Returns true if rect1 is fully contained in rect2
fn bounding_box_contained(rect1: Rect, rect2: Rect) -> bool {
    rect2.contains(rect1.max) && rect2.contains(rect1.min)
}

// https://stackoverflow.com/questions/4981866/quadtree-for-2d-collision-detection

const MAX_ENTITIES: usize = 4;
const MAX_LEVELS: usize = 12;

pub struct QuadTree {
    root_node: TreeNode,
}

impl QuadTree {
    pub fn new(screen_rect: Rect, entities: Vec<(Entity, Rect)>) -> Self {
        let mut root_node = TreeNode::new(screen_rect, 0);
        root_node.add_entities(entities);

        Self { root_node }
    }
}


struct TreeNode {
    contained_entities: Vec<(Entity, Rect)>,
    nodes: Vec<TreeNode>,
    rect: Rect,
    level: usize,
}

impl TreeNode {
    fn new(rect: Rect, level: usize) -> Self {
        Self {
            contained_entities: Vec::new(),
            nodes: Vec::with_capacity(4),
            rect,
            level,
        }
    }

    #[allow(unused)]
    fn traverse(&self, target_index: usize, cur_index: usize) -> (Option<&TreeNode>, usize) {
        if target_index == cur_index { return (Some(self), cur_index); }

        let mut next = cur_index + 1;

        for node in &self.nodes {
            let (opt, next_index) = node.traverse(target_index, next);
            next = next_index;
            match opt {
                option @ Some(_) => return (option, next),
                None => continue
            }
        }

        (None, next)
    }

    fn flatten_tree(&self) -> Vec<&TreeNode> {
        let mut vec = vec![self];

        for node in &self.nodes {
            vec.append(&mut node.flatten_tree())
        }

        vec
    }

    fn descendents(&self) -> Vec<&TreeNode> {
        let mut descendents = vec![];

        for i in 0..self.nodes.len() {
            descendents.push(&self.nodes[i]);
            descendents.append(&mut self.nodes[i].descendents())
        }
        descendents
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.contained_entities.is_empty()
    }

    fn get_overlapping_sub_tress(&self, entity: (Entity, Rect)) -> Vec<(Entity, Rect)> {
        let mut entities = Vec::new();
        let descendents = self.descendents();

        for node in descendents {
            if bounding_box_overlaps(node.rect, entity.1) {
                for e in &node.contained_entities {
                    entities.push(*e);
                }
            }
        }

        entities
    }

    fn add_entities(&mut self, entities: Vec<(Entity, Rect)>) {
        for entity in entities {
            self.add_entity(entity);
        }
    }

    fn add_entity(&mut self, entity: (Entity, Rect)) {
        // if the entity is not inside this node at all, discard it
        if !bounding_box_overlaps(entity.1, self.rect) { return; }

        // if this node has child nodes defer this entity to the child nodes
        // unless it is not contained completely by any of the child nodes
        if !self.nodes.is_empty() {
            if !self.defer_entity_to_child_node(entity) {
                self.contained_entities.push(entity);
            }
            return;
        }

        // if there is less than max entities in this node, or if this is the deepest level
        // add the new entity to this node, otherwise divide this node and defer all entities
        // in this node to its new child nodes
        if self.contained_entities.len() < MAX_ENTITIES || self.level == MAX_LEVELS {
            self.contained_entities.push(entity);
        } else {
            self.divide();
            if !self.defer_entity_to_child_node(entity) {
                self.contained_entities.push(entity);
            }
        }
    }

    fn defer_entity_to_child_node(&mut self, entity: (Entity, Rect)) -> bool {
        let mut deferred = false;
        for node in &mut self.nodes {
            if bounding_box_contained(entity.1, node.rect) {
                node.add_entity(entity);
                deferred = true;
                break;
            }
        }
        deferred
    }

    fn divide(&mut self) {
        // create sub rects

        let center = self.rect.center();
        let left_max = Vec2::new(self.rect.min.x, self.rect.max.y);
        let right_min = Vec2::new(self.rect.max.x, self.rect.min.y);
        let quad1 = Rect::from_corners(center, self.rect.max);
        let quad2 = Rect::from_corners(center, left_max);
        let quad3 = Rect::from_corners(center, self.rect.min);
        let quad4 = Rect::from_corners(center, right_min);

        self.nodes.push(TreeNode::new(quad1, self.level + 1));
        self.nodes.push(TreeNode::new(quad2, self.level + 1));
        self.nodes.push(TreeNode::new(quad3, self.level + 1));
        self.nodes.push(TreeNode::new(quad4, self.level + 1));

        let mut temp = Vec::new();
        for i in 0..self.contained_entities.len() {
            if !self.defer_entity_to_child_node(self.contained_entities[i]) {
                temp.push(self.contained_entities[i]);
            }
        }

        self.contained_entities = temp;
    }
}


