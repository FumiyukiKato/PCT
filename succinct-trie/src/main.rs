extern crate succinct_trie;
extern crate savefile;

use clap::{AppSettings, Clap};
use glob::glob;
use regex::Regex;

use std::{char, io::{BufRead, BufReader}};
use std::fs::File;

use succinct_trie::{config::K_NOT_FOUND};
use succinct_trie::trie::{Trie, TrajectoryHash};

#[derive(Clap)]
#[clap(version = "0.1", author = "Fumiyuki K. <fumilemon79@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets input file name. Server data
    #[clap(short, long, default_value = "input.csv")]
    server_input_file: String,

    /// Sets input file name. Client data
    #[clap(short, long, default_value = "input.csv")]
    client_input_file: String,

    /// Sets input file name. Client data
    #[clap(short, long, default_value = "3")]
    duration_of_exposure: String,

    /// mode normal|accurate|doe|doe_accurate
    #[clap(short, long, default_value = "3")]
    mode: String,
}

pub fn read_trajectory_hash_from_csv(filename: &str) -> Vec<Vec<u8>> {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut hash_vec = Vec::new();
    for line in reader.lines().into_iter() {
        if let Ok(hash) = line {
            let chars: Vec<char> = hash.chars().collect();
            let mut hash_bytes: Vec<u8> = Vec::with_capacity(hash.len()/2);
            for i in 0..(hash.len()/2) {
                hash_bytes.push(16*hex_to_num(chars[2*i]) + hex_to_num(chars[2*i+1]));
            }
            hash_vec.push(hash_bytes);
        }
    }
    hash_vec
}

fn hex_to_num(c: char) -> u8 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => panic!("invalid hex string"),
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut server_data = read_trajectory_hash_from_csv(opts.server_input_file.as_str());
    server_data.sort();
    let trie = Trie::new(&server_data);
    println!("server byte size {} byte", trie.byte_size());

    let re = Regex::new(r".+/client-(?P<theta_t>\d+)-(?P<theta_l>\d+)-(?P<client_id>\d+).+.csv").unwrap();
    let mut results = Vec::new();
    let th = TrajectoryHash::new(6, 17, 11);

    let count = 100;

    for entry in glob(format!("{}/*.csv", opts.client_input_file).as_str()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let filepath = path.to_str().unwrap();
                let caps = match re.captures(filepath) {
                    Some(c) => c,
                    None => break
                };
                let client_id: u32 = caps["client_id"].parse().unwrap();

                if client_id > count {
                    continue;
                }

                println!("start ... filepath {}, client_id {}", filepath, client_id);
                let client_data = read_trajectory_hash_from_csv(path.to_str().unwrap());
                match opts.mode.as_str() {
                    "normal" => {
                        let mut client_result = Vec::new();
                        let mut query_id: u32 = 0;
        
                        for key in client_data.iter() {
                            if trie.exact_search(key) != K_NOT_FOUND {
                                client_result.push((query_id, true));
                            } else {
                                client_result.push((query_id, false));
                            }
                            query_id += 1;
                        }
        
                        results.push((client_id, client_result));
                    },
                    "accurate" => {
                        let mut client_result = Vec::new();
                        let mut query_id = 0;
        
                        for key in client_data.iter() {
                            if trie.accurate_search(key, &th) {
                                client_result.push((query_id, true));
                            } else {
                                client_result.push((query_id, false));
                            }
                            query_id += 1;
                        }
        
                        results.push((client_id, client_result));
                    },
                    "doe" => {
                        let time_range: usize = opts.duration_of_exposure.parse().unwrap();
                        let result = trie.doe_search(time_range, &client_data);
                        results.push((client_id, vec![(0, result)]));
                    },
                    "doe_accurate" => {
                        let time_range: usize = opts.duration_of_exposure.parse().unwrap();
                        let result = trie.accurate_doe_search(time_range, &client_data, &th);
                        results.push((client_id, vec![(0, result)]));
                    },
                    _ => panic!("invalid mode parameter")
                }
            },
            Err(_) => panic!("failed to find path"),
        }
    }

    let result_file_name = match opts.mode.as_str() {
        "normal"       => "pct_resutls.bin",
        "accurate"     => "pct_acc_results.bin",
        "doe"          => "pct_doe_results.bin",
        "doe_accurate" => "pct_doe_acc_results.bin",
        _ => panic!("invalid mode parameter")
    };
    savefile::prelude::save_file(result_file_name, 0, &results).expect("failed to save");

    // let mut server_data = read_trajectory_hash_from_csv(opts.server_input_file.as_str());

    // server_data.sort();
    // let trie = Trie::new(&server_data);

    // let client_data = read_trajectory_hash_from_csv(opts.client_input_file.as_str());

    // println!("[searching]");
    // println!("server byte size {} byte", trie.byte_size());
    // let mut not_found = 0;
    // let mut found = 0;

    // for key in client_data.iter() {
    //     if trie.exact_search(key) != K_NOT_FOUND {
    //         found += 1;
    //     } else {
    //         not_found += 1;
    //     }
    // }
    // println!("Trie not found: {}, found: {}", not_found, found);

    println!("ok.")
    
}