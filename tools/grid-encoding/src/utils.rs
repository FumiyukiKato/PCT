use core::f64;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::{fs::File, u32};

pub struct Trajectory {
    time: u32,
    latitude: f64,
    longitude: f64,
}

impl Trajectory {
    fn deserialize_from_string_record(string_record: csv::StringRecord) -> Trajectory {
        Trajectory {
            time: string_record[0].parse().expect("time is invalid"),
            latitude: string_record[1].parse().expect("latitude is invalid"),
            longitude: string_record[2].parse().expect("longitude is invalid"),
        }
    }
}

pub fn read_trajectory_from_csv(filename: &str, has_header: bool, time: u32) -> (Vec<Trajectory>, f64, f64, f64, f64) {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(reader);

    let mut trajectories = Vec::new();
    let mut lng_max = -200.;
    let mut lat_max = -2000.;
    let mut lng_min = 200.;
    let mut lat_min = 200.;
    for result in csv_reader.records().into_iter() {
        let record = Trajectory::deserialize_from_string_record(result.expect("invalid record"));
        if record.time == time {
            if record.longitude > lng_max {
                lng_max = record.longitude
            }
            if record.longitude < lng_min {
                lng_min = record.longitude
            }
            if record.latitude > lat_max {
                lat_max = record.latitude
            }
            if record.latitude < lat_min {
                lat_min = record.latitude
            }
            trajectories.push(record);
        }
    }
    
    (trajectories, lng_max, lng_min, lat_max, lat_min)
}

pub fn prepare_grid_vectors(
    grid_vector_lng_max: f64,
    grid_vector_lng_min: f64,
    grid_vector_lat_max: f64,
    grid_vector_lat_min: f64,
    theta_l_lng: f64,
    theta_l_lat: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut grid_vector_lng = Vec::new();
    let mut curr_lng = grid_vector_lng_min;
    while curr_lng < grid_vector_lng_max {
        grid_vector_lng.push(curr_lng);
        curr_lng += theta_l_lng;
    }
    grid_vector_lng.push(curr_lng);

    let mut grid_vector_lat = Vec::new();
    let mut curr_lat = grid_vector_lat_min;
    while curr_lat < grid_vector_lat_max {
        grid_vector_lat.push(curr_lat);
        curr_lat += theta_l_lat;
    }
    grid_vector_lat.push(curr_lat);

    (grid_vector_lng, grid_vector_lat)
}

pub fn write_trajectory_hash_csv(filename: &str, trajectory_hashes: Vec<u32>) {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(filename)
        .expect("file open error");
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);
    for hash in trajectory_hashes {
        csv_writer
            .write_field(format!("{}", hash))
            .expect("write falied");
        csv_writer
            .write_record(None::<&[u8]>)
            .expect("write falied");
    }
    csv_writer.flush().expect("flush doesn't work");
}

pub fn bulk_encode(trajectories: Vec<Trajectory>, grid_vectors: &(Vec<f64>, Vec<f64>)) -> Vec<u32> {
    let mut result = Vec::with_capacity(trajectories.len() * 9);
    for trajectory in trajectories.iter() {
        result.extend(grid_encoding(trajectory, grid_vectors));
    }
    result
}

pub fn grid_encoding(trajectory: &Trajectory, grid_vectors: &(Vec<f64>, Vec<f64>)) -> Vec<u32> {
    let lng_idx = match grid_vectors
        .0
        .binary_search_by(|probe| probe.partial_cmp(&trajectory.longitude).unwrap())
    {
        Ok(idx) => idx,
        Err(idx) => {
            if idx == grid_vectors.0.len() || idx == 0 {
                idx
            } else {
                if (trajectory.longitude - grid_vectors.0[idx-1])
                    <= (grid_vectors.0[idx] - trajectory.longitude)
                {
                    idx - 1
                } else {
                    idx
                }
            }
        }
    } as i32;

    let lat_idx = match grid_vectors
        .1
        .binary_search_by(|probe| probe.partial_cmp(&trajectory.latitude).unwrap())
    {
        Ok(idx) => idx,
        Err(idx) => {
            if idx == grid_vectors.1.len() || idx == 0 {
                idx
            } else {
                if (trajectory.latitude - grid_vectors.1[idx-1])
                    <= (grid_vectors.1[idx] - trajectory.latitude)
                {
                    idx - 1
                } else {
                    idx
                }
            }
        }
    } as i32;

    let mut grids = Vec::with_capacity(9);
    for i in -1..2 {
        for j in -1..2 {
            if 0 <= lng_idx + i
                && lng_idx + i < grid_vectors.0.len() as i32
                && 0 <= lat_idx + j
                && lat_idx + j < grid_vectors.1.len() as i32
            {
                grids.push((grid_vectors.0.len() as i32 * (lat_idx + j) + (lng_idx + i)) as u32)
            }
        }
    }
    grids
}

#[cfg(test)]
mod tests {
    use crate::utils::{self, Trajectory};

    #[test]
    fn prepare_grid_vectors() {
        assert_eq!(
            utils::prepare_grid_vectors(100.0, 0.0, 100.0, 0.0, 15.0, 18.0),
            (vec![0.0, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0, 105.0], vec![0.0, 18.0, 36.0, 54.0, 72.0, 90.0, 108.0])
        );
    }

    #[test]
    fn grid_encoding() {
        let grid_vectors = utils::prepare_grid_vectors(100.0, 0.0, 100.0, 0.0, 15.0, 18.0);
        let trajectory = Trajectory { time: 1597881600, longitude: 10.000010, latitude: 38.000 };
        assert_eq!(
            utils::grid_encoding(&trajectory, &&grid_vectors),
            vec![8, 16, 24, 9, 17, 25, 10, 18, 26],
        );
    }
}
