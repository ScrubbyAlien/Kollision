use bevy::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ProfilerPlugin;
pub struct UpdateProfilerPlugin(bool);

impl Plugin for ProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Profiler::new());
    }
}

impl ProfilerPlugin {
    pub fn update_profiler(print: bool) -> UpdateProfilerPlugin {
        UpdateProfilerPlugin(print)
    }
}

impl Plugin for UpdateProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, add_update_profiler);
        app.add_systems(PreUpdate, begin_update_sample);
        if self.0 {
            app.add_systems(PostUpdate, end_update_sample_print);
        } else {
            app.add_systems(PostUpdate, end_update_sample);
        }
    }
}

pub const UPDATE: &str = "Update";

fn add_update_profiler(mut profiler: ResMut<Profiler>) {
    profiler.insert_sample_group(UPDATE);
}

fn begin_update_sample(mut profiler: ResMut<Profiler>) {
    profiler.begin_sample_in_group(UPDATE, UPDATE, 0);
}

fn end_update_sample_print(mut profiler: ResMut<Profiler>) {
    profiler.end_sample_in_group(UPDATE, true);
}

fn end_update_sample(mut profiler: ResMut<Profiler>) {
    profiler.end_sample_in_group(UPDATE, false);
}


#[derive(Resource)]
pub struct Profiler {
    pub sample_groups: HashMap<String, SampleGroup>,
}

impl Profiler {
    fn new() -> Profiler {
        Profiler { sample_groups: HashMap::new() }
    }

    pub fn get_sample_group(&self, name: &str) -> &SampleGroup {
        self.sample_groups.get(name).unwrap()
    }

    pub fn get_mut_sample_group(&mut self, name: &str) -> &mut SampleGroup {
        self.sample_groups.get_mut(name).unwrap()
    }

    pub fn insert_sample_group(&mut self, name: &str) {
        self.sample_groups.insert(name.to_string(), SampleGroup::new());
    }

    pub fn begin_sample_in_group(&mut self, group_name: &str, algorithm: &str, entities: i32) {
        self.get_mut_sample_group(group_name).begin_sample(algorithm, entities);
    }

    pub fn end_sample_in_group(&mut self, group_name: &str, print: bool) {
        self.get_mut_sample_group(group_name).end_sample(print);
    }
}

pub struct SampleGroup {
    // algo 50   100  150 ... (number of entities)
    // a    0.1  0.3  0.7 ...
    // b    0.05 0.07 0.08 ...
    // c    0.09 0.1  0.11 ...
    current_algorithm: String,
    current_entity_number: i32,
    begin_instant: Instant,
    sample_ongoing: bool,
    samples: HashMap<String, HashMap<i32, Vec<Duration>>>,
}

impl SampleGroup {
    fn new() -> SampleGroup {
        SampleGroup {
            current_algorithm: "".to_string(),
            current_entity_number: 0,
            begin_instant: Instant::now(),
            samples: HashMap::new(),
            sample_ongoing: false,
        }
    }

    fn begin_sample(&mut self, name: &str, entities: i32) {
        self.current_algorithm = name.to_string();
        self.current_entity_number = entities;
        self.begin_instant = Instant::now();
        self.sample_ongoing = true;
    }

    fn end_sample(&mut self, print: bool) {
        if !self.sample_ongoing { return; }

        if self.samples.contains_key(&self.current_algorithm) {
            if self.samples[&self.current_algorithm].contains_key(&self.current_entity_number) {
                // push new duration on existing algo + entity number combo
                self.samples
                    .get_mut(&self.current_algorithm).unwrap()
                    .get_mut(&self.current_entity_number).unwrap()
                    .push(self.begin_instant.elapsed());
            } else {
                // create new vec with durations to be appended to later
                let duration = vec![self.begin_instant.elapsed()];
                self.samples
                    .get_mut(&self.current_algorithm).unwrap()
                    .insert(self.current_entity_number, duration);
            }
        } else {
            // create new hashmap for new algorithm
            let duration = vec![self.begin_instant.elapsed()];
            let entity_number = HashMap::from([
                (self.current_entity_number, duration)
            ]);
            self.samples.insert(self.current_algorithm.to_string(), entity_number);
        }

        if print {
            let algorithm_padded = format!("{:20}", &self.current_algorithm);
            let entity_padded = format!("{:5}", self.current_entity_number);
            let duration = self.begin_instant.elapsed().as_nanos().to_string();
            println!("{}   {}   {:}", algorithm_padded, entity_padded, duration);
        }

        self.sample_ongoing = false;
    }

    pub fn get_averages(&self) -> HashMap<String, HashMap<i32, Duration>> {
        let mut averages: HashMap<String, HashMap<i32, Duration>> = HashMap::new();

        for algorithm in self.samples.iter() {
            let algo_name = algorithm.0;
            for entity_number in algorithm.1.iter() {
                let number = entity_number.0;
                let length = entity_number.1.len();
                let mut sum = Duration::from_nanos(0);
                for i in 0..length {
                    sum += entity_number.1[i];
                }
                let avg: Duration = sum / length as u32;
                if averages.contains_key(algo_name) {
                    averages.get_mut(algo_name).unwrap().insert(*number, avg);
                } else {
                    let mut map = HashMap::new();
                    map.insert(*number, avg);
                    averages.insert(algo_name.to_string(), map);
                }
            }
        }

        averages
    }
}

