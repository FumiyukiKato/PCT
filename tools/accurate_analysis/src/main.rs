#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate savefile;

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
            let trajectories = utils::read_trajectory_from_csv(opts.input_file.as_str(), true);
            let results = utils::accurate_quereis(trajectories, theta_t, theta_l);
            savefile::prelude::save_file("acc_resutls.bin", 0, &results).expect("failed to save");


            let duration_of_exposure = 3; // 30 minutes
            let trajectories_per_clients = utils::read_trajectories_per_clients(vec![opts.input_file.as_str()], true);
            let results = utils::doe_accurate_quereis(trajectories_per_clients, duration_of_exposure, theta_t, theta_l);
            savefile::prelude::save_file("acc_doe_resutls.bin", 0, &results).expect("failed to save");

            // let resutls: Vec<(u32, bool)> = savefile::prelude::load_file("acc_resutls.bin", 0).expect("failed to save");
            // println!("results {:?}", resutls);
        },
        "trunc" => {
            utils::truncate_trajectory_db();
        }
        _ => panic!("Invlid mode argument")
    }
}
