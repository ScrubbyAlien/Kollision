use std::cmp::max;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
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
        app.add_systems(PreStartup, add_update_profiler_table);
        app.add_systems(First, store_update_instant);
        if self.0 {
            app.add_systems(Last, record_update_duration);
        } else {
            // app.add_systems(Last, end_update_sample);
        }
    }
}

pub const UPDATE: &str = "Update";

fn add_update_profiler_table(mut profiler: ResMut<Profiler>) {
    let columns = vec!["Frame time".to_string()];
    let rows = vec!["Update".to_string()];
    profiler.create_table(UPDATE, columns, rows);
}

fn store_update_instant(mut profiler: ResMut<Profiler>) {
    profiler.store_instant(UPDATE);
}

fn record_update_duration(mut profiler: ResMut<Profiler>) {
    let duration = profiler.get_duration(UPDATE);
    profiler.record_cell_data_by_table_row_col_index(0, 0, 0, duration.as_nanos());
}

// fn end_update_sample(mut profiler: ResMut<Profiler>) {
//     profiler.end_sample_in_group(UPDATE, false);
// }


#[derive(Resource)]
pub struct Profiler {
    pub tables: Vec<Table>,
    pub table_names: Vec<String>,
    pub instants: HashMap<String, Instant>,
}

impl Profiler {
    fn new() -> Profiler {
        Profiler {
            tables: Vec::new(),
            table_names: Vec::new(),
            instants: HashMap::new(),
        }
    }

    pub fn store_instant(&mut self, instant: &str) {
        self.instants.insert(instant.to_string(), Instant::now());
    }

    pub fn get_duration(&mut self, instant: &str) -> Duration {
        self.instants.get(instant).unwrap().elapsed()
    }

    pub fn create_table(
        &mut self,
        table_name: &str,
        rows: Vec<String>,
        columns: Vec<String>,
    ) -> usize {
        let table_index = self.tables.len();
        self.tables.push(Table::new(columns, rows));
        self.table_names.push(table_name.to_string());

        table_index
    }

    pub fn get_table_ref(&self, table_name: &str) -> &Table {
        let index = self.table_names.iter().position(|s| { s == table_name }).unwrap();
        &self.tables[index]
    }

    pub fn record_cell_data(&mut self, table: &str, row: &str, column: &str, value: u128) {
        let table_index = self.table_names.iter().position(|s| { s == table }).unwrap();
        self.record_cell_data_by_table_index(table_index, row, column, value);
    }

    pub fn record_cell_data_by_table_index(&mut self, table: usize, row: &str, column: &str, value: u128) {
        self.tables[table].insert_value_in_cell(row, column, value);
    }

    pub fn record_cell_data_by_table_row_col_index(
        &mut self,
        table: usize,
        row: usize,
        column: usize,
        value: u128,
    ) {
        self.tables[table].insert_value_in_cell_by_indices(row, column, value);
    }

    pub fn write_to_csv(&self, table: &str, file_name: &str) -> std::io::Result<()> {
        let table_ref = self.get_table_ref(table);
        let averages = table_ref.get_averages();
        let mut file = File::create(format!("{}.csv", file_name))?;

        file.write_all(table.as_bytes())?;
        file.write_all(b",")?;

        // write all column names
        for col in table_ref.columns.clone() {
            let c = format!("{},", col);
            file.write_all(c.as_bytes())?;
        }
        file.write_all(b"\n")?;

        'rows: for (i, row) in averages.iter().enumerate() {
            // write the row name first, break if no row names left
            if i >= table_ref.rows.len() { break 'rows; }
            file.write_all(format!("{},", table_ref.rows[i]).as_bytes())?;

            'cols: for (i, cell) in row.iter().enumerate() {
                // write only if there are column names left
                if i >= table_ref.columns.len() { break 'cols; }
                file.write_all(format!("{},", cell).as_bytes())?;
            }

            file.write_all(b"\n")?;
        }
        println!("Successful write to {}.csv", file_name);
        Ok(())
    }
}

pub const COLUMNS: usize = 200;
pub const ROWS: usize = 10;

pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<String>,
    cells: [[[u128; 2]; COLUMNS]; ROWS],
}

impl Table {
    fn new(columns: Vec<String>, rows: Vec<String>) -> Self {
        Table {
            columns,
            rows,
            cells: [[[0, 0]; COLUMNS]; ROWS],
        }
    }

    pub fn insert_value_in_cell(&mut self, row: &str, column: &str, value: u128) {
        let row_index = self.rows.iter().position(|s| { s == row }).unwrap();
        let column_index = self.columns.iter().position(|s| { s == column }).unwrap();
        self.insert_value_in_cell_by_indices(row_index, column_index, value);
    }

    pub fn insert_value_in_cell_by_indices(
        &mut self,
        row_index: usize,
        column_index: usize,
        value: u128,
    ) {
        let mut cell = self.cells[row_index][column_index];
        cell[0] += 1; // increment number of times we have recorded data for this cell
        cell[1] += value; // increase by the value of this recording, so we can average later
        self.cells[row_index][column_index] = cell;
    }

    pub fn get_averages(&self) -> [[f64; COLUMNS]; ROWS] {
        let mut averages: [[f64; COLUMNS]; ROWS] = [[0.; COLUMNS]; ROWS];

        for (row, column) in self.cells.iter().enumerate() {
            for (col, cell) in column.iter().enumerate() {
                if cell[0] == 0 {
                    averages[row][col] = 0.;
                } else {
                    averages[row][col] = cell[1] as f64 / cell[0] as f64;
                }
            }
        }

        averages
    }
}

