use std::fs::File;
use std::io::{Write};
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Clone, Default, Debug)]
pub struct Clocker<'a> {
    data: HashMap<&'a str, Instant>,
    result: HashMap<&'a str, Duration>,
}

impl <'a>Clocker<'a>  {
    pub fn new() -> Self {
        Clocker::default()
    }
    
    pub fn set_and_start(&mut self, name: &'a str) {
        self.data.insert(name, Instant::now());
    }

    pub fn stop(&mut self, name: &'a str) {
        match self.data.get_mut(name) {
            Some(instant) => { 
                let end = instant.elapsed();
                self.result.insert(name, end);
            },
            None => { println!("[Clocker] error!! {} is not found", name); }
        }
    }

    pub fn show_all(&self) {
        for (name, duration) in self.result.iter() {
            println!("[Clocker] {}:  {}.{:016} seconds", name, duration.as_secs(), duration.subsec_nanos() / 1_000_000);
        }
    }

    pub fn to_string(&self) -> String {
        let mut res = String::new();
        for (name, duration) in self.result.iter() {
            res = format!("{}{:<30}:  {}.{:016} seconds\n", res, name, duration.as_secs(), duration.subsec_nanos() / 1_000_000);
        }
        res
    }
}

pub fn write_to_file(
    file_name: String,
    data_structure_type: String,
    central_data_file: String,
    query_data_file: String,
    threashould: usize,
    response_data_type: String,
    clocker: Clocker,
) {
    let mut file = File::create(file_name).unwrap();
    let clocker_result: String = format!(
r#"
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Basic data
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
data structure type       : {data_structure_type}
central data file         : {central_data_file}
query data file           : {query_data_file}
threashould               : {threashould}
response data type        : {response_data_type}
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Clocker data
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
{clocker_string}
-----------------------------------------------------------------------------
"# ,
        data_structure_type=data_structure_type,
        central_data_file=central_data_file,
        query_data_file=query_data_file,
        threashould=threashould,
        response_data_type=response_data_type,
        clocker_string=clocker.to_string()
    );

    file.write_all(clocker_result.as_bytes()).unwrap();
}

pub fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => format!("{}", n.as_secs()),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn query_id_from_u8(query_id: &[u8]) -> u64 {
    let mut array: [u8; 8] = [0; 8];
    array.copy_from_slice(query_id);
    u64::from_be_bytes(array)
}