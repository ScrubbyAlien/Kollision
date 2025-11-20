use std::time::Duration;
use bevy::prelude::*;

pub struct ExperimentPlugin {
    pub first: usize,
    pub step: usize,
    pub number_of_steps: usize,
    pub sample_duration: Duration,
}

impl Plugin for ExperimentPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ExperimentProgress>();
        app.insert_resource(ExperimentParameters::new(
            self.first,
            self.step,
            self.number_of_steps,
            self.sample_duration,
        ));
        app.add_systems(Update, progress_experiment);
    }
}

#[derive(Message)]
pub struct ExperimentProgress(pub(crate) String);

#[derive(Resource)]
pub struct ExperimentParameters {
    pub sample_sizes: [usize; 100],
    pub sample_sizes_as_str: Vec<String>,
    pub sample_index: usize,
    pub number_samples: usize,
    sample_duration: Duration,
    current_sample_progress: Duration,
}

impl ExperimentParameters {
    fn new(first: usize, step: usize, number_samples: usize, sample_duration: Duration) -> ExperimentParameters {
        let sample_sizes = generate_sample_sizes(first, step);
        let mut sample_sizes_as_str: Vec<String> = Vec::with_capacity(100);
        for sample_size in sample_sizes.iter() {
            sample_sizes_as_str.push(sample_size.to_string());
        }

        ExperimentParameters {
            sample_sizes,
            sample_sizes_as_str,
            sample_index: 0,
            number_samples,
            sample_duration,
            current_sample_progress: Duration::from_secs(0),
        }
    }

    pub fn current_sample_size(&self) -> usize {
        self.sample_sizes[self.sample_index]
    }

    pub fn current_sample_size_str(&self) -> String {
        self.sample_sizes_as_str[self.sample_index].clone()
    }

    pub fn max_samples_reached(&self) -> bool {
        self.sample_index >= self.number_samples
    }
}

fn generate_sample_sizes(first: usize, step: usize) -> [usize; 100] {
    let mut array: [usize; 100] = [0; 100];

    #[allow(clippy::needless_range_loop)]
    for i in 0..100 {
        array[i] = first + (step * i);
    }
    array
}

fn progress_experiment(
    mut parameters: ResMut<ExperimentParameters>,
    time: Res<Time>,
    mut writer: MessageWriter<ExperimentProgress>,
) {
    // todo: progress between different algorithms

    parameters.current_sample_progress += Duration::from_secs_f32(time.delta_secs());
    if parameters.current_sample_progress >= parameters.sample_duration {
        let prev_sample_size = parameters.current_sample_size_str();
        parameters.sample_index += 1;
        if parameters.max_samples_reached() {
            writer.write(ExperimentProgress(prev_sample_size));
            writer.write(ExperimentProgress("Done".to_string()));
            return;
        }
        parameters.current_sample_progress = Duration::from_secs(0);
        writer.write(ExperimentProgress(prev_sample_size));
    }
}