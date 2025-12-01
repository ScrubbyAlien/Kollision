mod ball;
mod profiler;
mod physics;
mod experiment;
mod capsule;
mod collider;
mod collision_algorithms;

use bevy::prelude::*;
use bevy::color::palettes::basic::*;
use std::time::{Duration, Instant};
use clap::Parser;
use rand::distr::StandardUniform;
use rand::Rng;

use experiment::*;
use physics::*;
use ball::*;
use capsule::*;
use collider::*;
use profiler::*;
use collision_algorithms::*;

const MIN_SIZE: f32 = 2.;
const MAX_SIZE: f32 = 10.;

const SPAWNING_RECT: Rect = Rect {
    min: Vec2 { x: 0., y: 0. },
    max: Vec2 { x: 800., y: 500. },
};

const NON_COLLIDED_COLOR: Srgba = GRAY;
const COLLIDED_COLOR: Srgba = RED;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Print step execution times and draw gizmos
    #[arg(short = 'D', long, default_value_t = false)]
    debug: bool,
    /// The starting step size
    #[arg(short, long, default_value_t = 50)]
    first: usize,
    /// Step increment size
    #[arg(short, long, default_value_t = 50)]
    step: usize,
    /// How many steps to run
    #[arg(short, long, default_value_t = 5)]
    number: usize,
    /// Maximum time for each step
    #[arg(short, long, default_value_t = 10.)]
    duration: f32,
    /// Min number of calculations that should be done for each step
    #[arg(short, long, default_value_t = 200)]
    min: usize,
}

fn main() {
    let args = Args::parse();

    App::new()
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .add_message::<CollisionMessage>()
        .add_plugins(DefaultPlugins)
        .add_plugins((ProfilerPlugin, UpdateProfilerPlugin))
        .add_plugins(ExperimentPlugin {
            first: args.first,
            step: args.step,
            variations: 4,
            number_of_steps: args.number,
            step_duration: Duration::from_secs_f32(args.duration),
            min_calcs_per_step: args.min,
            debug: args.debug,
        })
        .add_plugins((PhysicsPlugin, ColliderPlugin))
        .add_systems(Startup, (setup, add_balls, add_capsules).chain())
        .add_systems(PreUpdate, clear_balls.run_if(on_message::<ExperimentProgress>))
        .add_systems(
            Update, (
                add_balls.run_if(on_message::<ExperimentProgress>),
                detect_collisions,
                process_experiment_progress.run_if(on_message::<ExperimentProgress>),
            ).chain(),
        )
        .add_systems(
            PostUpdate, (
                resolve_collisions,
                write_to_csvs.run_if(on_message::<AppExit>)
            ).chain(),
        )
        .run();
}

#[derive(Resource)]
struct CollisionTableIndex(usize);
#[derive(Resource)]
struct QuadTreeTableIndex(usize);

fn setup(mut commands: Commands, mut profiler: ResMut<Profiler>, exp_params: Res<ExperimentParameters>) {
    commands.spawn(Camera2d);
    let algorithms: Vec<String> = vec![
        "None".to_string(),
        "PairDetection".to_string(),
        "PairBoundingBox".to_string(),
        "QuadTree".to_string()
    ];
    let samples = exp_params.relevant_samples();

    let index = profiler.create_table("Collision", algorithms, samples.clone());
    commands.insert_resource(CollisionTableIndex(index));

    let qt_index = profiler.create_table(
        "Quad Tree",
        vec!["Build time".to_string(), "Traversal time".to_string()],
        samples.clone(),
    );
    commands.insert_resource(QuadTreeTableIndex(qt_index));
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

#[allow(clippy::too_many_arguments)]
fn detect_collisions(
    circles: Query<(Entity, &CircleCollider, &BoxCollider), With<Ball>>,
    capsules: Query<(Entity, &CapsuleCollider, &BoxCollider), With<Capsule>>,
    exp_params: Res<ExperimentParameters>,
    table_index: Res<CollisionTableIndex>,
    quad_table_index: Res<QuadTreeTableIndex>,
    mut profiler: ResMut<Profiler>,
    mut collision_writer: MessageWriter<CollisionMessage>,
    window: Single<&Window>,
    gizmos: Gizmos,
) {
    if exp_params.variation_index == exp_params.number_variations { return; }
    let bottom_corner = Vec2::new(-window.width() / 2., -window.height() / 2.);
    let top_corner = Vec2::new(window.width() / 2., window.height() / 2.);
    let window_rect = Rect::from_corners(bottom_corner, top_corner);

    let elapsed = match exp_params.variation_index {
        1 => pair_detection(&circles, &capsules, &mut collision_writer),
        2 => pair_bounding_box(
            &circles,
            &capsules,
            &mut collision_writer,
            gizmos,
            exp_params.debug,
        ),
        3 => quad_tree(
            &circles,
            &capsules,
            &mut collision_writer,
            window_rect,
            &mut profiler,
            quad_table_index.0,
            exp_params.sample_index,
            gizmos,
            exp_params.debug,
        ),
        _ => no_algorithm(&circles, &capsules, &mut collision_writer)
    };

    profiler.record_cell_data_by_table_row_col_index(
        table_index.0,
        exp_params.variation_index,
        exp_params.sample_index,
        elapsed,
    );
}

fn resolve_collisions(
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

fn process_experiment_progress(
    profiler: Res<Profiler>,
    experiment_parameters: Res<ExperimentParameters>,
    collision_table_index: Res<CollisionTableIndex>,
    mut reader: MessageReader<ExperimentProgress>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for message in reader.read() {
        if experiment_parameters.debug {
            let sample_size = &experiment_parameters.sample_sizes_as_str[message.0];
            let nanos = profiler.tables[collision_table_index.0].get_averages()[message.1][message.0];
            let algo = &profiler.tables[collision_table_index.0].rows[message.1];
            println!("({algo}) Sample size: {:>5}  time: {nanos:>10.1} ns = {:>7.1} calcs per second",
                     sample_size,
                     1_000_000_000. / nanos
            );
        }

        if message.2 { // check if this is the last sample
            app_exit.write(AppExit::Success);
        }
    }
}

fn write_to_csvs(profiler: Res<Profiler>, start_up_instant: Res<StartupInstant>) {
    profiler.write_to_csv("Collision", "collision_times").unwrap();
    profiler.write_to_csv("Physics", "physics_times").unwrap();
    profiler.write_to_csv("Update", "update_times").unwrap();
    profiler.write_to_csv("Quad Tree", "quad_tree_times").unwrap();

    let elapsed = start_up_instant.0.elapsed().as_secs();
    let mins = elapsed / 60;
    let secs = elapsed % 60;

    println!("finished: {mins} minutes {secs} seconds");
}


pub struct UpdateProfilerPlugin;

impl Plugin for UpdateProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, add_update_profiler_table);
        app.add_systems(First, store_update_instant);
        app.add_systems(Last, record_update_duration);
    }
}

#[derive(Resource)]
struct UpdateTableInfo(usize, Instant);

fn add_update_profiler_table(
    mut commands: Commands,
    mut profiler: ResMut<Profiler>,
    exp_params: Res<ExperimentParameters>,
) {
    let columns = exp_params.relevant_samples();
    let algorithms: Vec<String> = vec![
        "None".to_string(),
        "PairDetection".to_string(),
        "PairBoundingBox".to_string(),
        "QuadTree".to_string()
    ];
    let index = profiler.create_table("Update", algorithms, columns);
    commands.insert_resource(UpdateTableInfo(index, Instant::now()))
}

fn store_update_instant(mut update_table_info: ResMut<UpdateTableInfo>) {
    update_table_info.1 = Instant::now();
}

fn record_update_duration(
    update_table_info: ResMut<UpdateTableInfo>,
    exp_params: Res<ExperimentParameters>,
    mut profiler: ResMut<Profiler>,
) {
    let elapsed = update_table_info.1.elapsed().as_nanos();
    profiler.record_cell_data_by_table_row_col_index(
        update_table_info.0,
        exp_params.variation_index,
        exp_params.sample_index,
        elapsed,
    );
}


