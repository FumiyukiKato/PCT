#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate savefile;

mod utils;
mod schema;
mod model;

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
            // theta_t = 17 (1.2m), theta_l = 22 (17min) とすると 
            // → 距離 (ルート2 * 1.2m)以内，時間17min以内　に対する近似条件になると想定している．
            // false positiveを0にするためには，正確には前後17min, 前後1.2mを範囲にする必要があるのでそうする．結構false negative起こりそう．
            // 範囲を半分にすれば，同じブロック内の "半分の長さ以上の距離にあるデータ" がfalse positiveになってしまう．

            // 時間は17minなのでsecond(UNIXEPOCH)に直すだけ
            let theta_t = 17*60;
            // NYの緯度経度だと (ref. https://vldb.gsi.go.jp/sokuchi/surveycalc/surveycalc/bl2stf.html)
            // NYだと赤道の0.75倍くらい，つまりtheta_t=17のとき，横の変化(longitude経度の変化)幅は実際には，0.9mくらい（縦(latitude緯度の変化)は1.2mで同じ）(https://wiki.openstreetmap.org/wiki/Zoom_levels)
            // 緯度 0.0000108 の変化で1.2m
            // 経度 0.0000108  の変化で0.9m
            // 間を取って0.
            let theta_l = 0.0000108;

            // let duration_of_exposure = 3; // 3 minutes

            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();

            let mut results = Vec::new();
            // let mut doe_results = Vec::new();
            for entry in glob(format!("{}/*.csv", opts.input_file).as_str()).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        let filepath = path.to_str().unwrap();
                        let caps = match re.captures(filepath) {
                            Some(c) => c,
                            None => break
                        };
                        let client_id: u32 = caps["client_id"].parse().unwrap();
                        println!("filepath {}, client_id {}", filepath, client_id);
                        let trajectories = utils::read_trajectory_from_csv(path.to_str().unwrap(), true);
                        let result = utils::accurate_quereis(&trajectories, theta_t, theta_l);
                        results.push((client_id, result));

                        // let doe_result = utils::doe_accurate_quereis_for_client(&trajectories, duration_of_exposure, theta_t, theta_l);
                        // doe_results.push((client_id, doe_result));
                    },
                    Err(_) => panic!("failed to find path"),
                }
            }
            savefile::prelude::save_file("acc_resutls.bin", 0, &results).expect("failed to save");
            // savefile::prelude::save_file("acc_doe_resutls.bin", 0, &doe_results).expect("failed to save");

            // let resutls: Vec<(u32, bool)> = savefile::prelude::load_file("acc_resutls.bin", 0).expect("failed to save");
            // println!("results {:?}", resutls);
        },
        "trunc" => {
            utils::truncate_trajectory_db();
        }
        _ => panic!("Invlid mode argument")
    }
}
