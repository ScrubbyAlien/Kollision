use std::time::Instant;
use bevy::prelude::*;
use crate::profiler::Profiler;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_gravity_and_velocity);
        app.add_systems(Startup, create_profiler_table);
    }
}


pub const GRAVITY: f32 = -9.82;

#[derive(Component)]
pub struct RigidBody {
    velocity: Vec3,
    gravity: f32,
    mass: f32,
}

impl RigidBody {
    pub fn new() -> Self {
        RigidBody {
            velocity: Vec3::new(0., 0., 0.),
            gravity: GRAVITY,
            mass: 1.,
        }
    }

    pub fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn gravity_scale(mut self, scale: f32) -> Self {
        self.gravity = GRAVITY * scale;
        self
    }
}

static mut TABLE_INDEX: usize = 0;

fn create_profiler_table(mut profiler: ResMut<Profiler>) {
    unsafe {
        TABLE_INDEX = profiler.create_table(
            "Physics",
            vec!["time".to_string()],
            vec!["g & v".to_string()],
        );
    }
}

fn apply_gravity_and_velocity(
    bodies: Query<(&mut Transform, &mut RigidBody)>,
    time: Res<Time>,
    /*mut profiler: ResMut<Profiler>*/
) {
    // let start = Instant::now();

    for (mut transform, mut body) in bodies {
        // apply gravity
        let gravity_vector = Vec3::new(0., body.gravity * time.delta_secs(), 0.);
        body.velocity += gravity_vector;

        // apply velocity
        let frame_diff = body.velocity * time.delta_secs();
        transform.translation += frame_diff;
    }

    // unsafe {
    //     let elapsed = start.elapsed().as_nanos();
    //     profiler.record_cell_data_by_table_row_col_index(TABLE_INDEX, 0, 0, elapsed);
    //     println!("{}", profiler.tables[TABLE_INDEX].get_averages()[0][0])
    // }
}
