mod utils;

use clap::{AppSettings, Clap};
use glob::glob;
use regex::Regex;

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
    #[clap(short, long, default_value = "1598486400")] // 2020/08/20 00:00:00 (UTC)
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

    /// target server|client
    #[clap(short, long)]
    target: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let time: u32 = opts.time.as_str().parse().unwrap();
    let grid_vectors = utils::prepare_grid_vectors(
        opts.theta_l_lng_max,
        opts.theta_l_lng_min,
        opts.theta_l_lat_max,
        opts.theta_l_lat_min,
        opts.theta_l_lng,
        opts.theta_l_lat,
    );
    match opts.target.as_str() {
        "server" => {
            let trajectories =
                utils::read_trajectory_from_csv(opts.input_file.as_str(), true, time);
            let hashed = utils::bulk_encode(trajectories, &grid_vectors);
            utils::write_trajectory_hash_csv(opts.output_file.as_str(), hashed);
        }
        "client" => {
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

                        let trajectories =
                            utils::read_trajectory_from_csv(opts.input_file.as_str(), true, time);
                        let hashed = utils::bulk_encode(trajectories, &grid_vectors);
                        utils::write_trajectory_hash_csv(
                            format!("{}-{}.csv", opts.output_file.as_str(), client_id).as_str(),
                            hashed,
                        );
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
        }
        _ => panic!("invalid target parameter"),
    }

    println!("ok.")
}
