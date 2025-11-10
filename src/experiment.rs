use bevy::prelude::*;

pub struct ExperimentPlugin(pub u32, pub u32);

impl Plugin for ExperimentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExperimentParameters::new(self.0, self.1));
    }
}


#[derive(Resource, Clone)]
pub struct ExperimentParameters {
    pub sample_sizes: [u32; 100],
}

// todo: this sample size stuff should be in separate resource struct, ex ExperimentParameters
impl ExperimentParameters {
    fn new(first: u32, step: u32) -> ExperimentParameters {
        ExperimentParameters {
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