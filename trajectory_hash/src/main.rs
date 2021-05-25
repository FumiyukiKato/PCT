mod utils;

use clap::{AppSettings, Clap};
use utils::MixType;

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
    /// Parameter for time
    #[clap(long, default_value = "27")]
    theta_t: u32,
    /// Parameter for location
    #[clap(long, default_value = "20")]
    theta_l: u32,
    /// Parameter for time period
    #[clap(short, long, default_value = "1597881600")] // 2020/08/20 00:00:00
    start_time: u32,
    /// Parameter for time period
    #[clap(short, long, default_value = "1599090600")] // 2020/09/03 00:00:00
    end_time: u32,

    /// Parameter for mixing type
    #[clap(short, long, default_value = "mix")]
    mix_type: String,

}

fn main() {
    let opts: Opts = Opts::parse();
    
    let trajectories = utils::read_trajectory_from_csv(opts.input_file.as_str(), true);
    let mix_type = match opts.mix_type.as_str() {
        "mix" => MixType::Mix,
        "seperate" => MixType::Seperate,
        _ => panic!("invalid option: mix_type")
    };
    let time_period = (opts.start_time, opts.end_time);
    let hashed = utils::bulk_encode(trajectories, mix_type, opts.theta_t, opts.theta_l, time_period);
    utils::write_trajectory_hash_csv(opts.output_file.as_str(), hashed);

    println!("ok.")
}