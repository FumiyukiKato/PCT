use crate::config::{position_t, level_t};

pub struct Cache {
    seq: Vec<position_t>,
    max_level: level_t
}

impl Cache {
    pub fn empty() -> Self {
        Cache { seq: Vec::<position_t>::new(), max_level: 0 }
    }
    
    pub fn get_pos(&self, i: level_t) -> position_t {
        self.seq[i]
    }

    pub fn get_max_level(&self) -> level_t {
        self.max_level
    }
}