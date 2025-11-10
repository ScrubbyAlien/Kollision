mod ball;
mod profiler;
mod physics;
mod experiment;
mod capsule;
mod collider;

use std::f32::consts::PI;
use bevy::prelude::*;
use profiler::ProfilerPlugin;
use crate::experiment::ExperimentParameters;
use rand::distr::StandardUniform;
use rand::Rng;

use ball::create_ball;
use crate::capsule::create_capsule;

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
        .add_plugins(experiment::ExperimentPlugin(50, 0))
        .add_plugins(physics::PhysicsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, (add_balls, add_capsules))
        // .add_systems(PostUpdate, print_average)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[allow(dead_code)]
fn print_average(profiler: Res<profiler::Profiler>, mut messages: MessageReader<AppExit>) {
    for _message in messages.read() {
        let averages = profiler.get_sample_group(profiler::UPDATE).get_averages();
        println!("average {}", averages[profiler::UPDATE][&0].as_micros());
    }
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

    for _i in 0..info.sample_sizes[0] {
        let tr: f32 = rng.sample(StandardUniform);
        let random_radius: f32 = MIN_SIZE + tr * (MAX_SIZE - MIN_SIZE);

        let tx: f32 = rng.sample(StandardUniform);
        let ty: f32 = rng.sample(StandardUniform);
        let random_x: f32 = x + SPAWNING_RECT.width() * tx;
        let random_y: f32 = y + SPAWNING_RECT.height() * ty;

        commands.spawn(create_ball(
            random_radius,
            Color::srgb(0.3, 0.3, 0.3),
            Transform::from_xyz(random_x, random_y, 0.),
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
    let pos_1 = vec3(0., -250., 0.);
    let pos_2 = vec3(0., -50., 0.);

    commands.spawn(create_capsule(
        200.,
        20.,
        Color::srgb(0., 0., 0.),
        pos_2,
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(create_capsule(
        500.,
        20.,
        Color::srgb(0., 0., 0.),
        pos_1,
        &mut meshes,
        &mut materials,
    ));
}
