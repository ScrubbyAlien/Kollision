mod ball;
mod profiler;
mod physics;
mod experiment;
mod capsule;
mod collider;

use std::time::{Duration, Instant};
use bevy::prelude::*;
use profiler::ProfilerPlugin;
use experiment::*;
use physics::*;
use rand::distr::StandardUniform;
use rand::Rng;

use ball::*;
use capsule::*;
use collider::*;
use profiler::Profiler;

const MIN_SIZE: f32 = 5.;
const MAX_SIZE: f32 = 10.;

const SPAWNING_RECT: Rect = Rect {
    min: Vec2 { x: 0., y: 0. },
    max: Vec2 { x: 400., y: 300. },
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .add_plugins(DefaultPlugins)
        .add_plugins((ProfilerPlugin, /*ProfilerPlugin::update_profiler(true)*/))
        .add_plugins(ExperimentPlugin {
            first: 100,
            step: 100,
            number_of_steps: 100,
            sample_duration: Duration::from_secs_f32(0.5),
            variations: 4,
        })
        .add_plugins(PhysicsPlugin)
        .add_plugins(ColliderPlugin)
        .add_systems(Startup, (setup, add_balls, add_capsules).chain())
        .add_systems(PreUpdate, clear_balls.run_if(on_message::<ExperimentProgress>))
        .add_systems(
            Update, (
                add_balls.run_if(on_message::<ExperimentProgress>),
                check_collision_with_capsules,
                store_profiling_data.run_if(on_message::<ExperimentProgress>),
            ).chain(),
        )
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

fn check_collision_with_capsules(
    circles: Query<(Entity, &CircleCollider, &MeshMaterial2d<ColorMaterial>), With<Ball>>,
    capsules: Query<&CapsuleCollider>,
    mut profiler: ResMut<Profiler>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    experiment_parameters: Res<ExperimentParameters>,
    table_index: Res<CollisionTableIndex>,
) {
    let start = Instant::now();

    for (e1, circle, material) in circles {
        let mat = materials.get_mut(material).unwrap();
        let mut collided = false;
        for capsule in capsules {
            if circle.collide_with_capsule(capsule) {
                collided = true;
            }
        }
        'circles: for (e2, other_circle, _) in circles {
            if e1.eq(&e2) { continue 'circles; }
            if circle.collide_with_circle(other_circle) {
                collided = true;
            }
        }
        if collided {
            mat.color = Color::srgb(1., 0., 0.);
        } else {
            mat.color = Color::srgb(0.3, 0.3, 0.3);
        }
    }
    let elapsed = start.elapsed().as_nanos();

    profiler.record_cell_data_by_table_row_col_index(
        table_index.0,
        experiment_parameters.variation_index,
        experiment_parameters.sample_index,
        elapsed,
    );
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
