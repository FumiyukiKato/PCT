#[macro_use]
extern crate diesel;
extern crate dotenv;

mod utils;
mod schema;
mod model;

use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "0.1", author = "Fumiyuki K. <fumilemon79@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets input file name. It should have trajectory data
    #[clap(short, long, default_value = "input.csv")]
    input_file: String,
    
    /// mode insert|query|trunc
    #[clap(short, long)]
    mode: String
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.mode.as_str() {
        "insert" => {
            let trajectories = utils::read_trajectory_from_csv(opts.input_file.as_str(), true);
            utils::store_trajectories(trajectories);
        },
        "query" => {
            let theta_t = 10000;
            let theta_l = 0.01;
            let trajectories = utils::read_trajectory_from_csv(opts.input_file.as_str(), true);
            let results = utils::accurate_quereis(trajectories, theta_t, theta_l);
            println!("{:?}", results);

            let duration_of_exposure = 3;
            let trajectories_per_clients = utils::read_trajectories_per_clients(vec![opts.input_file.as_str()], true);
            let results = utils::doe_accurate_quereis(trajectories_per_clients, duration_of_exposure, theta_t, theta_l);
            println!("{:?}", results);
        },
        "trunc" => {
            utils::truncate_trajectory_db();
        }
        _ => panic!("Invlid mode argument")
    }
}
