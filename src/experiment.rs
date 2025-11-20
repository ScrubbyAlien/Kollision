use bevy::prelude::*;

pub struct ExperimentPlugin(pub u32, pub u32);

impl Plugin for ExperimentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExperimentParameters::new(self.0, self.1));
    }
}


#[derive(Resource)]
pub struct ExperimentParameters {
    pub sample_sizes: [u32; 100],
    pub sample_sizes_as_str: Vec<String>,
}

impl ExperimentParameters {
    fn new(first: u32, step: u32) -> ExperimentParameters {
        let sample_sizes = generate_sample_sizes(first, step);
        let mut sample_sizes_as_str: Vec<String> = Vec::with_capacity(100);
        for sample_size in sample_sizes.iter() {
            sample_sizes_as_str.push(sample_size.to_string());
        }

        ExperimentParameters {
            sample_sizes,
            sample_sizes_as_str,
        }
    }
}

fn generate_sample_sizes(first: u32, step: u32) -> [u32; 100] {
    let mut array: [u32; 100] = [0; 100];

    #[allow(clippy::needless_range_loop)]
    for i in 0..100 {
        array[i] = first + (step * i as u32);

    }
    array
}