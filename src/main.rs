mod ball;
mod profiler;
mod physics;
mod experiment;
mod capsule;
mod collider;

use std::time::Instant;
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
        .add_plugins(ExperimentPlugin(50, 50))
        .add_plugins(PhysicsPlugin)
        .add_plugins(ColliderPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, (add_balls, add_capsules))
        .add_systems(Update, check_collision_with_capsules)
        // .add_systems(PostUpdate, print_average)
        .run();
}

fn setup(mut commands: Commands, mut profiler: ResMut<Profiler>, experiment_parameters: Res<ExperimentParameters>) {
    commands.spawn(Camera2d);
    let algorithms: Vec<String> = vec![
        "None".to_string(),
        "PairPruning".to_string(),
        "BoundingBox".to_string(),
        "QuadTree".to_string()
    ];
    profiler.create_table("Collision", algorithms, experiment_parameters.sample_sizes_as_str.clone());
}

fn add_balls(
    mut commands: Commands,
    window: Single<&Window>,
    info: Res<ExperimentParameters>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    let x = -SPAWNING_RECT.width() / 2.;
    let y = (window.height() / 2.) - SPAWNING_RECT.height();

    for _i in 0..info.sample_sizes[9] { // 50 * (9 + 1)
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
    circles: Query<(&CircleCollider, &MeshMaterial2d<ColorMaterial>), With<Ball>>,
    capsules: Query<&CapsuleCollider>,
    mut profiler: ResMut<Profiler>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let start = Instant::now();

    for (circle, material) in circles {
        let mat = materials.get_mut(material).unwrap();
        let mut collided = false;
        for capsule in capsules {
            if circle.collide_with_capsule(capsule) {
                collided = true;
            }
        }
        if collided {
            mat.color = Color::srgb(1., 0., 0.);
        } else {
            mat.color = Color::srgb(0.3, 0.3, 0.3);
        }
    }

    profiler.record_cell_data("Collision", "None", "500", start.elapsed().as_nanos());
    let nanos = profiler.get_table_ref("Collision").get_averages()[0][9];
    println!("{:.1} ns: {:.1} calcs per second", nanos, 1_000_000_000. / nanos)



}
