extern crate savefile;

mod utils;

use clap::{AppSettings, Clap};
use glob::glob;
use regex::Regex;
use std::collections::HashSet;

#[derive(Clap)]
#[clap(version = "0.1", author = "Fumiyuki K. <fumilemon79@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets input file name. It should have trajectory data
    #[clap(short, long, default_value = "input.csv")]
    input_file: String,
    /// Sets output file name. Output is trajectoryhashed data.
    #[clap(short, long, default_value = "output.csv")]
    output_file: String,

    /// Parameter for time period
    #[clap(long, default_value = "1598486400")] // 2020/08/20 00:00:00 (UTC)
    time: String,

    /// Parameter for location
    #[clap(long)]
    theta_l_lng: f64,

    /// Parameter for location
    #[clap(long)]
    theta_l_lat: f64,

    /// Parameter for location
    #[clap(long)]
    theta_l_lng_max: f64,

    /// Parameter for location
    #[clap(long)]
    theta_l_lng_min: f64,

    /// Parameter for location
    #[clap(long)]
    theta_l_lat_max: f64,

    /// Parameter for location
    #[clap(long)]
    theta_l_lat_min: f64,

    /// target server|client|pct
    #[clap(long)]
    target: String,

    /// Sets output file name. Output is trajectoryhashed data.
    #[clap(long)]
    client_input_dir: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();
	let count = 100;

    let time: u32 = opts.time.as_str().parse().unwrap();
    let grid_vectors = utils::prepare_grid_vectors(
        - opts.theta_l_lng_max,
        - opts.theta_l_lng_min,
        opts.theta_l_lat_max,
        opts.theta_l_lat_min,
        opts.theta_l_lng,
        opts.theta_l_lat,
    );
	
    match opts.target.as_str() {
        "server" => {
            let (trajectories, lng_max, lng_min, lat_max, lat_min) =
                utils::read_trajectory_from_csv(opts.input_file.as_str(), true, time);
			println!("server data size: {}, lng_max: {}, lng_min: {}, lat_max: {}, lat_min: {}", trajectories.len(), lng_max, lng_min, lat_max, lat_min); 
            let hashed = utils::bulk_encode(trajectories, &grid_vectors);
            utils::write_trajectory_hash_csv(opts.output_file.as_str(), hashed);
        }
        "client" => {
		    let mut lng_max_mut = -200.;
            let mut lat_max_mut = -200.;
            let mut lng_min_mut = 200.;
            let mut lat_min_mut = 200.;
	
            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();

            for entry in glob(format!("{}/*.csv", opts.input_file).as_str())
                .expect("Failed to read glob pattern")
            {
                match entry {
                    Ok(path) => {
                        let filepath = path.to_str().unwrap();
                        let caps = match re.captures(filepath) {
                            Some(c) => c,
                            None => break,
                        };
                        let client_id: u32 = caps["client_id"].parse().unwrap();
						if count <= client_id {
							continue;
						}
                        let (trajectories, lng_max, lng_min, lat_max, lat_min) =
                            utils::read_trajectory_from_csv(path.to_str().unwrap(), true, time);
						if lng_max > lng_max_mut {
							lng_max_mut = lng_max
						}
						if lng_min < lng_min_mut {
							lng_min_mut = lng_min
						}
						if lat_max > lat_max_mut {
							lat_max_mut = lat_max
						}
						if lat_min < lat_max_mut {
							lat_min_mut = lat_min
						}
                        let hashed = utils::bulk_encode(trajectories, &grid_vectors);
                        println!("client data size: {}", hashed.len());
                        utils::write_trajectory_hash_csv(
                            format!("{}-{}.csv", opts.output_file.as_str(), client_id).as_str(),
                            hashed,
                        );
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
			println!("lng_max: {}, lng_min: {}, lat_max: {}, lat_min: {}", lng_max_mut, lng_min_mut, lat_max_mut, lat_min_mut);
        }
        "pct" => {
            // server-side data
            let (trajectories, _, _, _, _) =
            utils::read_trajectory_from_csv(opts.input_file.as_str(), true, time);
            let hashed = utils::bulk_encode(trajectories, &grid_vectors);
            let server_data: HashSet<u32> = hashed.into_iter().collect();

            // clie t-side data
            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();
            let client_input_dir = opts.client_input_dir.as_ref().unwrap().as_str();

            let mut results = Vec::new();

            for entry in glob(format!("{}/*.csv", client_input_dir).as_str())
                .expect("Failed to read glob pattern")
            {
                match entry {
                    Ok(path) => {
                        let filepath = path.to_str().unwrap();
                        let caps = match re.captures(filepath) {
                            Some(c) => c,
                            None => break,
                        };
                        let client_id: u32 = caps["client_id"].parse().unwrap();
						if count <= client_id {
							continue;
						}
                        let (trajectories, _, _, _, _) =
                            utils::read_trajectory_from_csv(path.to_str().unwrap(), true, time);
                        let hashed = utils::bulk_encode(trajectories, &grid_vectors);

                        let mut client_result = Vec::new();
                        let mut query_id: u32 = 0;
                        for hash in hashed {
                            client_result.push((query_id, server_data.contains(&hash)));
                            query_id += 1;
                        }
                        results.push((client_id, client_result));
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
            savefile::prelude::save_file(opts.output_file.as_str(), 0, &results)
                .expect("failed to save");
        },
        _ => panic!("invalid target parameter"),
    }

    println!("ok.")
}
