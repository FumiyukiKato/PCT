use primitive::*;
use std::vec::Vec;
use constant::*;

/* Type Period */
#[derive(Clone, Default, Debug)]
pub struct Period(UnixEpoch, UnixEpoch);

impl Period {
    pub fn new(start: UnixEpoch, end: UnixEpoch) -> Self {
        Period(start, end)
    }

    pub fn with_start(start: UnixEpoch) -> Self {
        Period(start, start)
    }

    pub fn start(&self) -> UnixEpoch {
        self.0
    }

    pub fn from_unixepoch_vector(unixepoch_vec: &Vec<UnixEpoch>) -> Vec<Period> {
        let mut period_vec: Vec<Period> = vec![];
        
        assert!(unixepoch_vec.len() > 0);
        let mut latest_unixepoch: UnixEpoch = unixepoch_vec[0];
        let mut period = Period::with_start(latest_unixepoch);
        
        for unixepoch in unixepoch_vec.iter() {
            if latest_unixepoch + TIME_INTERVAL >= *unixepoch {
                latest_unixepoch = *unixepoch;
            } else {
                period.1 = latest_unixepoch;
                period_vec.push(period);
                period = Period::with_start(*unixepoch);
                latest_unixepoch = *unixepoch;
            }
        }
        period.1 = latest_unixepoch;
        period_vec.push(period);
        period_vec
    }

    // period - CONTACT_TIME_THREASHOLD < unixepoch < period + CONTACT_TIME_THREASHOLD
    pub fn is_include(&self, unixepoch: UnixEpoch) -> bool {
        self.0 - CONTACT_TIME_THREASHOLD < unixepoch && unixepoch < self.1 + CONTACT_TIME_THREASHOLD
    }
}