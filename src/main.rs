mod ball;
mod profiler;
mod physics;
mod experiment;
mod capsule;
mod collider;
mod collision_algorithms;

use bevy::prelude::*;
use bevy::color::palettes::basic::*;
use std::time::{Duration};
use rand::distr::StandardUniform;
use rand::Rng;

use experiment::*;
use physics::*;
use ball::*;
use capsule::*;
use collider::*;
use profiler::*;
use collision_algorithms::*;

const MIN_SIZE: f32 = 5.;
const MAX_SIZE: f32 = 10.;

const SPAWNING_RECT: Rect = Rect {
    min: Vec2 { x: 0., y: 0. },
    max: Vec2 { x: 400., y: 300. },
};

const NON_COLLIDED_COLOR: Srgba = GRAY;
const COLLIDED_COLOR: Srgba = RED;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .add_message::<CollisionMessage>()
        .add_plugins(DefaultPlugins)
        .add_plugins((ProfilerPlugin, /*ProfilerPlugin::update_profiler(true)*/))
        .add_plugins(ExperimentPlugin {
            first: 100,
            step: 100,
            number_of_steps: 5,
            sample_duration: Duration::from_secs_f32(5.),
            variations: 2,
        })
        .add_plugins(PhysicsPlugin)
        .add_plugins(ColliderPlugin)
        .add_systems(Startup, (setup, add_balls, add_capsules).chain())
        .add_systems(PreUpdate, clear_balls.run_if(on_message::<ExperimentProgress>))
        .add_systems(
            Update, (
                add_balls.run_if(on_message::<ExperimentProgress>),
                check_collisions,
                store_profiling_data.run_if(on_message::<ExperimentProgress>),
            ).chain(),
        )
        .add_systems(PostUpdate, affect_collision)
        .run();
}

#[derive(Resource)]
struct CollisionTableIndex(usize);

fn setup(mut commands: Commands, mut profiler: ResMut<Profiler>, exp_params: Res<ExperimentParameters>) {
    commands.spawn(Camera2d);
    let algorithms: Vec<String> = vec![
        "None".to_string(),
        "PairDetection".to_string(),
        "BoundingBox".to_string(),
        "QuadTree".to_string()
    ];
    let sample_slice = Vec::from(&exp_params.sample_sizes_as_str[..exp_params.number_samples]);
    let index = profiler.create_table("Collision", algorithms, sample_slice);
    commands.insert_resource(CollisionTableIndex(index));
}

fn add_balls(
    mut commands: Commands,
    window: Single<&Window>,
    experiment_parameters: Res<ExperimentParameters>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    let x = -SPAWNING_RECT.width() / 2.;
    let y = (window.height() / 2.) - SPAWNING_RECT.height();
    let size = experiment_parameters.current_sample_size();

    for _i in 0..size {
        let tr: f32 = rng.sample(StandardUniform);
        let random_radius: f32 = MIN_SIZE + tr * (MAX_SIZE - MIN_SIZE);

        let tx: f32 = rng.sample(StandardUniform);
        let ty: f32 = rng.sample(StandardUniform);
        let random_x: f32 = x + SPAWNING_RECT.width() * tx;
        let random_y: f32 = y + SPAWNING_RECT.height() * ty;

        commands.spawn(create_ball(
            random_radius,
            Color::srgb(0.3, 0.3, 0.3),
            Transform::from_xyz(random_x, random_y, 1.),
            &mut meshes,
            &mut materials,
        ));
    }
}

fn clear_balls(
    balls: Query<Entity, With<Ball>>,
    mut commands: Commands,
) {
    for ball in balls.iter() {
        commands.entity(ball).despawn();
    }
}

fn add_capsules(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(create_capsule(
        200.,
        20.,
        Color::srgb(0., 0., 0.),
        vec3(0., -50., 0.),
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(create_capsule(
        500.,
        20.,
        Color::srgb(0., 0., 0.),
        vec3(0., -250., 0.),
        &mut meshes,
        &mut materials,
    ));
}

fn check_collisions(
    circles: Query<(Entity, &CircleCollider), With<Ball>>,
    capsules: Query<(Entity, &CapsuleCollider), With<Capsule>>,
    experiment_parameters: Res<ExperimentParameters>,
    table_index: Res<CollisionTableIndex>,
    mut profiler: ResMut<Profiler>,
    mut collision_writer: MessageWriter<CollisionMessage>,
) {
    let elapsed = match experiment_parameters.variation_index {
        1 => pair_detection(&circles, &capsules, &mut collision_writer),
        2 => bounding_box(&circles, &capsules, &mut collision_writer),
        3 => quad_tree(&circles, &capsules, &mut collision_writer),
        _ => no_algorithm(&circles, &capsules, &mut collision_writer)
    };

    profiler.record_cell_data_by_table_row_col_index(
        table_index.0,
        experiment_parameters.variation_index,
        experiment_parameters.sample_index,
        elapsed,
    );
}

fn affect_collision(
    mut collisions: MessageReader<CollisionMessage>,
    // mut transforms: Query<(&mut Transform, &mut RigidBody)>,
    mut materials_asset: ResMut<Assets<ColorMaterial>>,
    material_comps: Query<&MeshMaterial2d<ColorMaterial>, With<Ball>>,
) {
    for ball_mat in material_comps {
        materials_asset.get_mut(ball_mat).unwrap().color = Color::from(NON_COLLIDED_COLOR);
    }


    for collision in collisions.read() {
        if let Ok(mat) = material_comps.get(collision.entity1) {
            materials_asset.get_mut(mat).unwrap().color = Color::from(COLLIDED_COLOR);
        }
        if let Ok(mat) = material_comps.get(collision.entity2) {
            materials_asset.get_mut(mat).unwrap().color = Color::from(COLLIDED_COLOR);
        }
    }
}

fn store_profiling_data(
    profiler: Res<Profiler>,
    experiment_parameters: Res<ExperimentParameters>,
    collision_table_index: Res<CollisionTableIndex>,
    mut reader: MessageReader<ExperimentProgress>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for message in reader.read() {
        let sample_size = &experiment_parameters.sample_sizes_as_str[message.0];
        let nanos = profiler.tables[collision_table_index.0].get_averages()[message.1][message.0];
        let algo = &profiler.tables[collision_table_index.0].rows[message.1];
        println!("({algo}) Sample size: {:>5}  time: {nanos:>10.1} ns = {:>7.1} calcs per second",
                 sample_size,
                 1_000_000_000. / nanos
        );

        if message.2 { // check if this is the last sample
            profiler.write_to_csv("Collision", "collision_times").unwrap();
            app_exit.write(AppExit::Success);
        }
    }
}
