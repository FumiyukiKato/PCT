use crate::schema::trajectory;

#[derive(Queryable)]
pub struct TrajectoryModel {
    pub id: i32,
    pub time: u32,
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Insertable)]
#[table_name = "trajectory"]
pub struct Trajectory {
    time: i64,
    longitude: f64,
    latitude: f64,
}

impl Trajectory {
    pub fn deserialize_from_string_record(string_record: csv::StringRecord) -> Trajectory {
        Trajectory {
            time: string_record[0].parse().expect("time is invalid"),
            latitude: string_record[1].parse().expect("latitude is invalid"),
            longitude: string_record[2].parse().expect("longitude is invalid"),
        }
    }

    pub fn get_query_condition(&self, theta_t: i64, theta_l: f64) -> (i64, i64, f64, f64, f64, f64) {
        (
            self.time - (theta_t/2),
            self.time + (theta_t/2),
            self.longitude - (theta_l/2.0),
            self.longitude + (theta_l/2.0),
            self.latitude - (theta_l/2.0),
            self.latitude + (theta_l/2.0),
        )
    }
}