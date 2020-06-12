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
}