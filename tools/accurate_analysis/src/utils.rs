use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

use std::fs::File;
use std::io::BufReader;

use crate::model::Trajectory;
use crate::schema::trajectory;

pub fn read_trajectories_per_clients(filenames: Vec<&str>, has_header: bool) -> Vec<Vec<Trajectory>> {
    let mut trajectories_per_clients = Vec::new();
    for filename in filenames {
        trajectories_per_clients.push(read_trajectory_from_csv(filename, has_header));
    }
    trajectories_per_clients
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

pub fn store_trajectories(trajectories: Vec<Trajectory>) -> () {
    let connection = establish_connection();

    for trajectory in trajectories {
        create_trajectory(&connection, trajectory);
    }
}

pub fn doe_accurate_quereis(
    trajectories_per_clients: Vec<Vec<Trajectory>>,
    duration_of_exposure: i64,
    theta_t: i64,
    theta_l: f64,
) -> Vec<(u32, bool)> {
    let connection = establish_connection();
    let mut results = Vec::new();
    let mut client_id = 0;
    for trajectories_for_client in trajectories_per_clients {
        results.push((
            client_id,
            doe_accurate_quereis_for_client(
                &connection,
                trajectories_for_client,
                duration_of_exposure,
                theta_t,
                theta_l,
            ),
        ));
        client_id += 1;
    }
    results
}

fn doe_accurate_quereis_for_client(
    connection: &SqliteConnection,
    trajectories: Vec<Trajectory>,
    duration_of_exposure: i64,
    theta_t: i64,
    theta_l: f64,
) -> bool {
    let mut seq_count = 0;

    for trajectory in trajectories {
        let (time_start, time_end, lng_start, lng_end, lat_start, lat_end) =
            trajectory.get_query_condition(theta_t, theta_l);
        let ret = query_contact_detection(
            &connection,
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
    trajectories: Vec<Trajectory>,
    theta_t: i64,
    theta_l: f64,
) -> Vec<(u32, bool)> {
    let connection = establish_connection();
    let mut results = Vec::new();

    let mut query_id = 0;
    for trajectory in trajectories {
        let (time_start, time_end, lng_start, lng_end, lat_start, lat_end) =
            trajectory.get_query_condition(theta_t, theta_l);
        results.push((
            query_id,
            query_contact_detection(
                &connection,
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

pub fn truncate_trajectory_db() {
    let connection = establish_connection();
    diesel::delete(trajectory::table)
        .execute(&connection)
        .expect("falied to delete");
}

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = "trajectory.db";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn create_trajectory(conn: &SqliteConnection, traj: Trajectory) -> usize {
    diesel::insert_into(trajectory::table)
        .values(&traj)
        //SQLiteはget_result()は対応していないため、execute()
        .execute(conn)
        .expect("Error saving new post")
}

// accurate query
fn query_contact_detection(
    conn: &SqliteConnection,
    time_start: i64,
    time_end: i64,
    lng_start: f64,
    lng_end: f64,
    lat_start: f64,
    lat_end: f64,
) -> bool {
    let all_ids = trajectory::table
        .select(trajectory::id)
        .filter(trajectory::time.between(time_start, time_end))
        .filter(trajectory::longitude.between(lng_start, lng_end))
        .filter(trajectory::latitude.between(lat_start, lat_end))
        .load::<i32>(conn)
        .expect("falied to query");
    all_ids.len() > 0
}
