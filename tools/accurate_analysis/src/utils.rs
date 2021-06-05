use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

use std::fs::File;
use std::io::BufReader;

use crate::model::Trajectory;
use crate::schema::trajectory;

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

pub fn read_trajectory_from_csv_by_time(filename: &str, has_header: bool, time: i64) -> Vec<Trajectory> {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(reader);

    let mut trajectories = Vec::new();
    for result in csv_reader.records().into_iter() {
        let record = Trajectory::deserialize_from_string_record(result.expect("invalid record"));
        if record.get_time() == time {
            trajectories.push(record);
        }
    }
    trajectories
}

pub fn store_trajectories(trajectories: Vec<Trajectory>) -> () {
    let connection = establish_connection();

    diesel::insert_into(trajectory::table)
        .values(trajectories)
        //SQLiteはget_result()は対応していないため、execute()
        .execute(&connection)
        .expect("Error saving new post");
}

pub fn doe_accurate_quereis_for_client(
    trajectories: &Vec<Trajectory>,
    duration_of_exposure: i64,
    theta_t: i64,
    theta_l_lng: f64,
    theta_l_lat: f64,
) -> bool {
    let connection = establish_connection();
    let mut seq_count = 0;
    let ratio = theta_l_lng / theta_l_lat;
    let threshold  = theta_l_lng.powi(2);

    for trajectory in trajectories {
        let (time_start, time_end, lng_start, lng_end, lat_start, lat_end) =
            trajectory.get_query_condition(theta_t, theta_l_lng, theta_l_lat);
        let ret = query_contact_detection(
            &connection,
            &trajectory,
            ratio,
            threshold,
            time_start,
            time_end,
            lng_start,
            lng_end,
            lat_start,
            lat_end,
        );
        if ret {
            seq_count += 1;
            if seq_count >= duration_of_exposure {
                return true;
            }
        } else {
            seq_count = 0;
        }
    }
    return false;
}

pub fn accurate_quereis(
    trajectories: &Vec<Trajectory>,
    theta_t: i64,
    theta_l_lng: f64,
    theta_l_lat: f64,
) -> Vec<(u32, bool)> {
    let connection = establish_connection();
    let mut results = Vec::new();
    let ratio = theta_l_lng / theta_l_lat;
    let threshold  = theta_l_lng.powi(2);

    let mut query_id: u32 = 0;
    for trajectory in trajectories {
        let (time_start, time_end, lng_start, lng_end, lat_start, lat_end) =
            trajectory.get_query_condition(theta_t, theta_l_lng, theta_l_lat);
        results.push((
            query_id,
            query_contact_detection(
                &connection,
                &trajectory,
                ratio,
                threshold,
                time_start,
                time_end,
                lng_start,
                lng_end,
                lat_start,
                lat_end,
            ),
        ));
        query_id += 1;
    }
    results
}

pub fn obliv_accurate_quereis(
    trajectories: &Vec<Trajectory>,
    theta_t: i64,
    theta_l_lng: f64,
    theta_l_lat: f64,
) -> Vec<(u32, bool)> {
    let connection = establish_connection();
    let mut results = Vec::new();
    let ratio = theta_l_lng / theta_l_lat;
    let threshold  = theta_l_lng.powi(2);

    let mut query_id: u32 = 0;
    for trajectory in trajectories {
        let (_, _, lng_start, lng_end, lat_start, lat_end) =
            trajectory.get_query_condition(theta_t, theta_l_lng, theta_l_lat);
        results.push((
            query_id,
            obliv_query_contact_detection(
                &connection,
                &trajectory,
                ratio,
                threshold,
                lng_start,
                lng_end,
                lat_start,
                lat_end,
            ),
        ));
        query_id += 1;
    }
    results
}

pub fn truncate_trajectory_db() {
    let connection = establish_connection();
    diesel::delete(trajectory::table)
        .execute(&connection)
        .expect("falied to delete");
}

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = "trajectory-rand.db";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// fn create_trajectory(conn: &SqliteConnection, traj: Trajectory) -> usize {
//     diesel::insert_into(trajectory::table)
//         .values(&traj)
//         //SQLiteはget_result()は対応していないため、execute()
//         .execute(conn)
//         .expect("Error saving new post")
// }

// accurate query
fn query_contact_detection(
    conn: &SqliteConnection,
    trajectory: &Trajectory,
    ratio: f64,
    threshold: f64,
    time_start: i64,
    time_end: i64,
    lng_start: f64,
    lng_end: f64,
    lat_start: f64,
    lat_end: f64,
) -> bool {
    let all_data = trajectory::table
        .select((trajectory::longitude, trajectory::latitude))
        .filter(trajectory::time.between(time_start, time_end))
        .filter(trajectory::longitude.between(lng_start, lng_end))
        .filter(trajectory::latitude.between(lat_start, lat_end))
        .load::<(f64, f64)>(conn)
        .expect("falied to query");
    
    if all_data.len() == 0 {
        return false
    }
    
    // |-|-|-|
    // |-|-|-|
    // |-|-|-|
    // NY
    // lng 0.0000215
    // lat 0.0000165
    // Kinki
    // lng 0.0000215
    // lat 0.0000175
    // Tokyo
    // lng 0.0000215
    // lat 0.0000174
    for (lng, lat) in all_data {
        if (trajectory.longitude - lng).powi(2) + ((trajectory.latitude - lat)*ratio).powi(2) <= threshold {
            return true
        }
    }
    return false
}

// accurate query
fn obliv_query_contact_detection(
    conn: &SqliteConnection,
    trajectory: &Trajectory,
    ratio: f64,
    threshold: f64,
    lng_start: f64,
    lng_end: f64,
    lat_start: f64,
    lat_end: f64,
) -> bool {
    let all_data = trajectory::table
        .select((trajectory::longitude, trajectory::latitude))
        .filter(trajectory::time.eq(trajectory.time))
        .filter(trajectory::longitude.between(lng_start, lng_end))
        .filter(trajectory::latitude.between(lat_start, lat_end))
        .load::<(f64, f64)>(conn)
        .expect("falied to query");
        if all_data.len() == 0 {
            return false
        }
        for (lng, lat) in all_data {
            if (trajectory.longitude - lng).powi(2) + ((trajectory.latitude - lat)*ratio).powi(2) <= threshold {
                return true
            }
        }
        return false
}
