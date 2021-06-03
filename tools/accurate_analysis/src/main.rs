#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate savefile;

mod model;
mod schema;
mod utils;

use clap::{AppSettings, Clap};
use glob::glob;
use regex::Regex;

use std::{collections::HashMap, iter::FromIterator};

#[derive(Clap)]
#[clap(version = "0.1", author = "Fumiyuki K. <fumilemon79@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets input file name. It should have trajectory data
    #[clap(short, long)]
    input_file: Option<String>,

    /// Sets output file name. binary data
    #[clap(short, long, default_value = "acc_results.bin")]
    output_file: String,

    /// mode insert|query|trunc|doe|obliv
    #[clap(short, long)]
    mode: Option<String>,

    /// result-analysis or db-access (default: db-access)
    #[clap(short, long, default_value = "db-access")]
    usecase: String,

    /// result file name
    #[clap(short, long)]
    accurate_result_file: Option<String>,

    /// result file name
    #[clap(short, long)]
    pct_result_file: Option<String>,

    /// theta-t minutes
    #[clap(short, long)]
    theta_t: Option<String>,

    /// theta_l_lng longitude
    #[clap(short, long)]
    theta_l_lng: Option<String>,

    /// theta_l_lat latitude
    #[clap(short, long)]
    theta_l_lat: Option<String>,
}

fn result_analysis(opts: &Opts) {
    let accurate_result_file_name = opts.accurate_result_file.as_ref().unwrap().as_str();
    let pct_result_file_name = opts.pct_result_file.as_ref().unwrap().as_str();

    let acc_resutls: Vec<(u32, Vec<(u32, bool)>)> =
        savefile::prelude::load_file(accurate_result_file_name, 0).expect("failed to load");
    let pct_resutls: Vec<(u32, Vec<(u32, bool)>)> =
        savefile::prelude::load_file(pct_result_file_name, 0).expect("failed to load");

    // calculate confusion_matrix
    let mut confusion_matrix = HashMap::new();
    confusion_matrix.insert("tp", 0);
    confusion_matrix.insert("fp", 0);
    confusion_matrix.insert("fn", 0);
    confusion_matrix.insert("tn", 0);

    let clinet_map: HashMap<u32, Vec<(u32, bool)>> = HashMap::from_iter(pct_resutls);

    for acc_result_per_client in acc_resutls {
        let pct_result_per_client = &clinet_map[&acc_result_per_client.0];
        for (acc_result, pct_result) in acc_result_per_client.1.iter().zip(pct_result_per_client) {
            if acc_result.0 != pct_result.0 {
                panic!("query_id is different!")
            }
            if acc_result.1 == true && pct_result.1 == true {
                *confusion_matrix.get_mut("tp").unwrap() += 1;
            } else if acc_result.1 == false && pct_result.1 == true {
                *confusion_matrix.get_mut("fp").unwrap() += 1;
            } else if acc_result.1 == true && pct_result.1 == false {
                *confusion_matrix.get_mut("fn").unwrap() += 1;
            } else {
                *confusion_matrix.get_mut("tn").unwrap() += 1;
            }
        }
    }
    println!("{:?}", confusion_matrix);
}

fn db_access(opts: &Opts) {
    let mode = opts.mode.as_ref().unwrap().as_str();
    match mode {
        "insert" => {
            let input_file = opts.input_file.as_ref().unwrap().as_str();
            let trajectories = utils::read_trajectory_from_csv(input_file, true);
            utils::store_trajectories(trajectories);
        }
        "query" => {
            // theta_l = 24 (2.4m) (https://docs.microsoft.com/ja-jp/azure/azure-maps/zoom-levels-and-tile-grid?tabs=csharp), theta_t = 22 (17min) とすると
            // → 距離 (ルート2 * m)以内，時間17min以内　に対する近似条件になると想定している．
            // false positiveを0にするためには，正確には前後17min, 前後1.2mを範囲にする必要があるのでそうする．結構false negative起こりそう．
            // 範囲を半分にすれば，同じブロック内の "半分の長さ以上の距離にあるデータ" がfalse positiveになってしまう．

            // 時間は17minなのでsecond(UNIXEPOCH)に直すだけ
            let theta_t: i64 = opts.theta_t.as_ref().unwrap().as_str().parse().unwrap();
            let theta_t = theta_t * 60;
            // let theta_t = 17*60;

            // NYの緯度経度だと (ref. https://vldb.gsi.go.jp/sokuchi/surveycalc/surveycalc/bl2stf.html)
            // 緯度はNYだと赤道の0.75倍くらい，(https://wiki.openstreetmap.org/wiki/Zoom_levels)
            // さらにタイル座標ではタイルのサイズが大体正方形なので下のようなquadkeyのサイズ設定は下の感じになる
            // 経度(lng) 0.0000215の変化で1.836mでハッシュ値が1変化
            // 緯度(lat) 0.0000165 の変化で1.832mでハッシュ値が１変化
            // ちなみに 最大距離は約2.6m
            let theta_l_lng: f64 = opts.theta_l_lng.as_ref().unwrap().as_str().parse().unwrap();
            let theta_l_lat: f64 = opts.theta_l_lat.as_ref().unwrap().as_str().parse().unwrap();
            // let theta_l_lng = 0.0000215;
            // let theta_l_lat = 0.0000165;

            // let duration_of_exposure = 3; // 3 minutes

            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();
            let count = 100;

            let mut results = Vec::new();
            // let mut doe_results = Vec::new();
            let input_file = opts.input_file.as_ref().unwrap().as_str();
            for entry in
                glob(format!("{}/*.csv", input_file).as_str()).expect("Failed to read glob pattern")
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
                        println!("filepath {}, client_id {}", filepath, client_id);
                        let trajectories =
                            utils::read_trajectory_from_csv(path.to_str().unwrap(), true);
                        let result = utils::accurate_quereis(
                            &trajectories,
                            theta_t,
                            theta_l_lng,
                            theta_l_lat,
                        );
                        results.push((client_id, result));
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
            savefile::prelude::save_file(opts.output_file.as_str(), 0, &results)
                .expect("failed to save");
        }
        "doe" => {
            let theta_t: i64 = opts.theta_t.as_ref().unwrap().as_str().parse().unwrap();
            let theta_t = theta_t * 60;
            let theta_l_lng: f64 = opts.theta_l_lng.as_ref().unwrap().as_str().parse().unwrap();
            let theta_l_lat: f64 = opts.theta_l_lat.as_ref().unwrap().as_str().parse().unwrap();
            let duration_of_exposure = 15; // 51 minutes

            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();
            let count = 100;

            let mut doe_results = Vec::new();
            let input_file = opts.input_file.as_ref().unwrap().as_str();
            for entry in
                glob(format!("{}/*.csv", input_file).as_str()).expect("Failed to read glob pattern")
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
                        println!("filepath {}, client_id {}", filepath, client_id);
                        let trajectories =
                            utils::read_trajectory_from_csv(path.to_str().unwrap(), true);
                        let doe_result = utils::doe_accurate_quereis_for_client(
                            &trajectories,
                            duration_of_exposure,
                            theta_t,
                            theta_l_lng,
                            theta_l_lat,
                        );
                        doe_results.push((client_id, vec![(0u32, doe_result)]));
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
            savefile::prelude::save_file(opts.output_file.as_str(), 0, &doe_results)
                .expect("failed to save");
        }
        "trunc" => {
            utils::truncate_trajectory_db();
        }
        "obliv" => {
            let theta_t: i64 = opts.theta_t.as_ref().unwrap().as_str().parse().unwrap();
            let theta_l_lng: f64 = opts.theta_l_lng.as_ref().unwrap().as_str().parse().unwrap();
            let theta_l_lat: f64 = opts.theta_l_lat.as_ref().unwrap().as_str().parse().unwrap();

            let re = Regex::new(r".+/client-(?P<client_id>\d+)-.+.csv").unwrap();
            let count = 100;

            let mut results = Vec::new();
            let input_file = opts.input_file.as_ref().unwrap().as_str();
            for entry in
                glob(format!("{}/*.csv", input_file).as_str()).expect("Failed to read glob pattern")
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
                        println!("filepath {}, client_id {}", filepath, client_id);
                        let trajectories =
                            utils::read_trajectory_from_csv_by_time(path.to_str().unwrap(), true, theta_t);
                        let result = utils::obliv_accurate_quereis(
                            &trajectories,
                            theta_t,
                            3.0*theta_l_lng,
                            3.0*theta_l_lat,
                        );
                        results.push((client_id, result));
                    }
                    Err(_) => panic!("failed to find path"),
                }
            }
            savefile::prelude::save_file(opts.output_file.as_str(), 0, &results)
                .expect("failed to save");
        }
        _ => panic!("Invlid mode argument"),
    }
}
fn main() {
    let opts: Opts = Opts::parse();

    match opts.usecase.as_str() {
        "db-access" => db_access(&opts),
        "result-analysis" => result_analysis(&opts),
        _ => panic!("invalid usecase"),
    }
}
