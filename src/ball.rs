use bevy::prelude::*;
// #[cfg(not(target_arch = "wasm32"))]
// use bevy::sprite_render::{Wireframe2dConfig, Wireframe2dPlugin};
use rand::distr::StandardUniform;
use rand::Rng;

// min and max size for the number of balls
pub struct BallPlugin {
    pub min_size: f32,
    pub max_size: f32,
    pub first_sample: u32,
    pub sample_step: u32,
}

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        let info = SpawningInformation::new(
            self.min_size,
            self.max_size,
            self.first_sample,
            self.sample_step,
        );
        app.insert_resource(info);
        app.add_systems(Startup, add_balls);
    }
}

#[derive(Component)]
struct Ball {
    radius: f32,
    color: Color,
}

#[derive(Resource, Clone)]
struct SpawningInformation {
    min_size: f32,
    max_size: f32,
    sample_sizes: [u32; 100],
}

impl SpawningInformation {
    fn new(min: f32, max: f32, first: u32, step: u32) -> SpawningInformation {
        SpawningInformation {
            min_size: min,
            max_size: max,
            sample_sizes: generate_sample_sizes(first, step),
        }
    }
}
fn generate_sample_sizes(first: u32, step: u32) -> [u32; 100] {
    let mut array: [u32; 100] = [0; 100];
    for i in 0..100 {
        array[i] = first + step * i as u32;
    }
    array
}

const NUMBER_OF_BALLS: u32 = 10;

fn add_balls(
    mut commands: Commands,
    window: Single<&Window>,
    info: Res<SpawningInformation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for i in 0..info.sample_sizes[0] {
        let mut rng = rand::rng();
        let tr: f32 = rng.sample(StandardUniform);
        let random_radius: f32 = info.min_size + tr * (info.min_size - info.max_size);

        let tx: f32 = rng.sample(StandardUniform);
        let ty: f32 = rng.sample(StandardUniform);
        let random_x: f32 = window.width() * (tx - 0.5);
        let random_y: f32 = window.height() * (ty - 0.5);

        commands.spawn(create_ball(
            random_radius,
            Color::srgb(0., 0., 0.),
            Transform::from_xyz(random_x, random_y, 0.),
            &mut meshes,
            &mut materials,
        ));
    }
}

fn create_ball(
    radius: f32,
    color: Color,
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Ball, Mesh2d, MeshMaterial2d<ColorMaterial>, Transform)
{
    let ball = Ball { radius, color };
    let mesh = meshes.add(Circle::new(ball.radius));
    let material = materials.add(ball.color);

    (
        ball,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        transform
    )
}




