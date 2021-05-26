extern crate succinct_trie;

use clap::{AppSettings, Clap};

use std::{char, io::{BufRead, BufReader}};
use std::fs::File;

use succinct_trie::{config::K_NOT_FOUND, trie::TrajectoryHash};
use succinct_trie::trie::Trie;

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

    let client_data = read_trajectory_hash_from_csv(opts.client_input_file.as_str());

    println!("[searching]");
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

    let time_range: usize = opts.duration_of_exposure.parse().unwrap();
    let result = trie.doe_search(time_range, &client_data);
    println!("Result: {}", result);

    // let th = TrajectoryHash::new(7, 20, 16);
    // println!("{:?}", &th.mask_lists);
    // let neis = trie.get_neighbors(&0b11000001011001011001111000000111100101110101010001110u128.to_be_bytes().to_vec()[9..], &th);
    // println!("{:?}", neis.len());
    // println!("{:?}", neis);

    println!("done.")
    
}