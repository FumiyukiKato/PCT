use std::{env, fs::File, io::Write, path::Path};

fn main () {
    let out_dir = env::var("OUT_DIR").expect("No out dir");
    let dest_path = Path::new(&out_dir).join("init_constants.rs");
    let mut f = File::create(&dest_path).expect("Could not create file");

    let encoded_value_size = option_env!("ENCODEDVALUE_SIZE");
    let encoded_value_size: usize = encoded_value_size
        .expect("Could not parse ENCODEDVALUE_SIZE")
        .parse()
        .expect("Could not parse ENCODEDVALUE_SIZE");
    write!(&mut f, "pub const ENCODEDVALUE_SIZE: usize = {};", encoded_value_size)
        .expect("Could not write file");

    let query_size = option_env!("QUERY_SIZE");
    let query_size: usize = query_size
        .expect("Could not parse QUERY_SIZE")
        .parse()
        .expect("Could not parse QUERY_SIZE");
    write!(&mut f, "pub const QUERY_SIZE: usize = {};", query_size)
        .expect("Could not write file");
}
