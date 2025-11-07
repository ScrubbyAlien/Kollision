mod ball;
mod profiler;

use bevy::prelude::*;
use profiler::ProfilerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((ProfilerPlugin, /*ProfilerPlugin::update_profiler(false)*/))
        .add_plugins(ball::BallPlugin {
            min_size: 10.,
            max_size: 30.,
            first_sample: 30,
            sample_step: 0,
        })
        .add_systems(Startup, setup)
        // .add_systems(PostUpdate, print_average)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn print_average(profiler: Res<profiler::Profiler>, mut messages: MessageReader<AppExit>) {
    for _message in messages.read() {
        let averages = profiler.get_sample_group(profiler::UPDATE).get_averages();
        println!("average {}", averages[profiler::UPDATE][&0].as_micros());
    }
}
