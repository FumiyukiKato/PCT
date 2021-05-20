use std::io::{BufReader};
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

pub fn read_trajectory_from_csv(filename: &str, has_header: bool) -> Vec<Trajectory> {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(reader);

    let mut trajectories = Vec::new();
    for result in csv_reader.records().into_iter() {
        let record = Trajectory::deserialize_from_string_record(result.expect("invalid record"));
        trajectories.push(record);
    }
    trajectories
}
