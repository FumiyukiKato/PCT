use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::{f64::consts::PI, fs::File, u32};

const MAX_LONGITUDE: f64 = 180.0;
const MAX_LATITUDE: f64 = 85.05112877980659; // (2*math.atan2(exp(math.pi))*180.0/math.pi - 90.0)
const MAX_ZOOM: u32 = 31;
const MAX_THETA_T: u32 = 32;
const MIN_LONGITUDE: f64 = -MAX_LONGITUDE;
const MIN_LATITUDE: f64 = -MAX_LATITUDE;

static mut PRINT_FLAG: bool = true;

#[derive(Debug)]
pub struct Trajectory {
    time: u32,
    latitude: f64,
    longitude: f64,
}

pub enum MixType {
    Seperate,
    Mix,
}

impl Trajectory {
    fn deserialize_from_string_record(string_record: csv::StringRecord, format: i32) -> Trajectory {
        match format {
            1 => Trajectory {
                time: string_record[0].parse().expect("time is invalid"),
                latitude: string_record[1].parse().expect("latitude is invalid"),
                longitude: string_record[2].parse().expect("longitude is invalid"),
            },
            2 => Trajectory {
                time: string_record[0].parse().expect("time is invalid"),
                longitude: string_record[1].parse().expect("longitude is invalid"),
                latitude: string_record[2].parse().expect("latitude is invalid"),
            },
            _ => panic!("invalid format parameter")
        }
    }

    fn serialize_to_string_record(hash: Vec<u8>) -> String {
        hex::encode(hash)
    }
}

pub fn read_trajectory_from_csv(filename: &str, has_header: bool, format: i32) -> Vec<Trajectory> {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .from_reader(reader);

    let mut trajectories = Vec::new();
    for result in csv_reader.records().into_iter() {
        let record = Trajectory::deserialize_from_string_record(result.expect("invalid record"), format);
        trajectories.push(record);
    }
    trajectories
}

pub fn write_trajectory_hash_csv(filename: &str, trajectory_hashes: Vec<Vec<u8>>) {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(filename)
        .expect("file open error");
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(writer);
    for hash in trajectory_hashes {
        csv_writer
            .write_field(Trajectory::serialize_to_string_record(hash))
            .expect("write falied");
        csv_writer
            .write_record(None::<&[u8]>)
            .expect("write falied");
    }
    csv_writer.flush().expect("flush doesn't work");
}

pub fn bulk_encode(
    trajectories: Vec<Trajectory>,
    mix_type: &MixType,
    theta_t: u32,
    theta_l: u32,
    time_period: (u32, u32),
) -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(trajectories.len());
    for trajectory in trajectories.iter() {
        result.push(trajectory_hash(
            trajectory,
            &mix_type,
            theta_t,
            theta_l,
            time_period,
        ));
    }
    result
}

pub fn trajectory_hash(
    trajectory: &Trajectory,
    mix_type: &MixType,
    theta_t: u32,
    theta_l: u32,
    time_period: (u32, u32),
) -> Vec<u8> {
    assert!(theta_l < MAX_ZOOM, "theta_l has to be  less");
    assert!(theta_t <= MAX_THETA_T, "theta_t has to be less");

    let (b1, b2, geo_length) = quadkey_encoding(trajectory.longitude, trajectory.latitude, theta_l);
    let (b3, time_length) = periodical_encoding(trajectory.time, time_period, theta_t);

    let (mixed, bit_length) = mix(mix_type, b1, b2, geo_length, b3, time_length);
    unsafe {
        if PRINT_FLAG {
            println!(
                "geo_length {}, time_length {} bit_length {}",
                geo_length, time_length, bit_length
            );
            PRINT_FLAG = false;
        }
    }
    base8_encoding(&mixed, bit_length)
}

fn mix(
    mix_type: &MixType,
    b1: u32,
    b2: u32,
    geo_length: u32,
    b3: u32,
    time_length: u32,
) -> (u128, u32) {
    let geo_bit_mask = u32::MAX >> (32 - geo_length);
    let time_bit_mask = u32::MAX >> (32 - time_length);
    return match mix_type {
        MixType::Seperate => {
            let mut mixed = 0u128;
            let b1 = (b1 & geo_bit_mask) as u128;
            let b2 = (b2 & geo_bit_mask) as u128;
            let b3 = (b3 & time_bit_mask) as u128;

            let digit = 2 * geo_length + time_length;
            for i in 0..geo_length {
                mixed |= (b1 & (1u128 << (geo_length - i - 1))) << (digit - i - geo_length);
                mixed |= (b2 & (1u128 << (geo_length - i - 1))) << (digit - 1 - i - geo_length);
            }
            mixed |= b3 as u128;
            (mixed, 2 * geo_length + time_length)
        }
        MixType::Mix => {
            let mut mixed = 0u128;
            let b1 = (b1 & geo_bit_mask) as u128;
            let b2 = (b2 & geo_bit_mask) as u128;
            let b3 = (b3 & time_bit_mask) as u128;

            let mut geo_cursor = geo_length;
            let mut time_cursor = time_length;
            let mut curr_digit = 2 * geo_length + time_length;

            while geo_cursor > time_cursor {
                if geo_cursor > 0 {
                    mixed |= (b1 & (1u128 << (geo_cursor - 1))) << (curr_digit - geo_cursor);
                    mixed |= (b2 & (1u128 << (geo_cursor - 1))) << (curr_digit - 1 - geo_cursor);
                    curr_digit -= 2;
                    geo_cursor -= 1;
                }
            }
            while geo_cursor < time_cursor {
                if time_cursor > 0 {
                    mixed |= (b3 & (1u128 << (time_cursor - 1))) << (curr_digit - time_cursor);
                    curr_digit -= 1;
                    time_cursor -= 1;
                }
            }
            while geo_cursor > 0 || time_cursor > 0 {
                if geo_cursor > 0 {
                    mixed |= (b1 & (1u128 << (geo_cursor - 1))) << (curr_digit - geo_cursor);
                    mixed |= (b2 & (1u128 << (geo_cursor - 1))) << (curr_digit - 1 - geo_cursor);
                    curr_digit -= 2;
                    geo_cursor -= 1;
                }
                if time_cursor > 0 {
                    mixed |= (b3 & (1u128 << (time_cursor - 1))) << (curr_digit - time_cursor);
                    curr_digit -= 1;
                    time_cursor -= 1;
                }
            }
            (mixed, 2 * geo_length + time_length)
        }
    };
}

fn base8_encoding<'a>(mixed: &u128, bit_length: u32) -> Vec<u8> {
    let base8_start = (16 - ((bit_length - 1) / 8 + 1)) as usize;
    let bytes = mixed.to_be_bytes();
    bytes[base8_start..].to_vec()
}

fn get_time_max_length(time_period: (u32, u32)) -> u32 {
    let t_diff = time_period.1 - time_period.0;
    let max_digit = 32 - stable_ctlz(t_diff);
    max_digit
}

fn stable_ctlz(num: u32) -> u32 {
    if num == 0 {
        return 32u32;
    }
    for i in 1..32 {
        if num >> i == 0u32 {
            return 32 - i;
        }
    }
    return 0u32;
}

fn periodical_encoding(time: u32, time_period: (u32, u32), theta_t: u32) -> (u32, u32) {
    // assert!(time >= time_period.0);
    // assert!(time <= time_period.1);
    if time < time_period.0 {
        println!("small time: {}", time);
    }
    if time > time_period.1 {
        println!("high time: {}", time);
    }
    let t_diff = time - time_period.0;
    let shift = 32 - theta_t;
    let zoomed_t_diff = t_diff >> shift;
    let time_length = get_time_max_length(time_period);
    (zoomed_t_diff, time_length - shift)
}

fn quadkey_encoding(lon: f64, lat: f64, zoom: u32) -> (u32, u32, u32) {
    let corrected_lon = min_f64(MAX_LONGITUDE, max_f64(MIN_LONGITUDE, lon));
    let corrected_lat = min_f64(MAX_LATITUDE, max_f64(MIN_LATITUDE, lat));

    // TransformToPixelCoodinate
    let fx = (corrected_lon + 180.0) / 360.0;
    let sinlat = (corrected_lat * PI / 180.0).sin();
    let fy = 0.5 - ((1.0 + sinlat) / (1.0 - sinlat)).ln() / (4.0 * PI);

    // 2**zoom
    let mapsize: u64 = 256 << zoom;

    let x = (fx * (mapsize as f64)).floor() as u64;
    let y = (fy * (mapsize as f64)).floor() as u64;

    let corrected_x = std::cmp::min(mapsize - 1, std::cmp::max(0, x)) / 256;
    let corrected_y = std::cmp::min(mapsize - 1, std::cmp::max(0, y)) / 256;

    (corrected_x as u32, corrected_y as u32, zoom)
}

fn min_f64(a: f64, b: f64) -> f64 {
    if a > b {
        return b;
    } else {
        return a;
    }
}

fn max_f64(a: f64, b: f64) -> f64 {
    if a < b {
        return b;
    } else {
        return a;
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{self, Trajectory};

    #[test]
    fn stable_ctlz_test() {
        assert_eq!(utils::stable_ctlz(u32::MAX), 0);
        assert_eq!(utils::stable_ctlz(u32::MIN), 32);
        assert_eq!(utils::stable_ctlz(1209600), 11);
        assert_eq!(utils::stable_ctlz(10000000), 8);
    }

    #[test]
    fn min_max_test() {
        assert_eq!(utils::max_f64(0.3, 1.5), 1.5);
        assert_eq!(utils::max_f64(1.5, 0.3), 1.5);
        assert_eq!(utils::min_f64(0.3, 1.5), 0.3);
        assert_eq!(utils::min_f64(1.5, 0.3), 0.3);
    }

    #[test]
    fn quadkey_encoding_test() {
        assert_eq!(
            utils::quadkey_encoding(139.759556, 35.716701, 20),
            (931367, 412778, 20)
        );
    }

    #[test]
    fn periodical_encoding_test() {
        assert_eq!(
            utils::periodical_encoding(1598555555, (1597849200, 1599058800), 27),
            (22073, 16)
        );
    }

    #[test]
    fn base8_encoding_test() {
        assert_eq!(
            utils::base8_encoding(&(0b0000000000110110110110 as u128), 12),
            vec![13, 182]
        );
        assert_eq!(
            utils::base8_encoding(&(0b0000110110110110 as u128), 12),
            vec![13, 182]
        );
        assert_eq!(
            utils::base8_encoding(
                &(0b11111111111111111111111111111111111111110000110110110110 as u128),
                40
            ),
            vec![255, 255, 255, 13, 182]
        );
    }

    #[test]
    fn mix_test() {
        assert_eq!(
            utils::mix(&utils::MixType::Mix, 0b1010u32, 0b1010u32, 4, 0b1010u32, 4),
            (0b111000111000 as u128, 4 + 4 + 4)
        );
        assert_eq!(
            utils::mix(
                &utils::MixType::Mix,
                0b1010u32,
                0b1010u32,
                4,
                0b0000001010u32,
                10
            ),
            (0b000000111000111000 as u128, 4 + 4 + 10)
        );
        assert_eq!(
            utils::mix(
                &utils::MixType::Mix,
                0b1010u32,
                0b1010u32,
                4,
                0b0000001010u32,
                3
            ),
            (0b11000111000 as u128, 4 + 4 + 3)
        );
        assert_eq!(
            utils::mix(
                &utils::MixType::Seperate,
                0b1010u32,
                0b1010u32,
                4,
                0b1010u32,
                4
            ),
            (0b110011001010 as u128, 4 + 4 + 4)
        );
    }

    #[test]
    fn trajectory_hash() {
        let trajectory = Trajectory {
            time: 1598555555,
            longitude: 139.759556,
            latitude: 35.716701,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                27,
                20,
                (1597849200, 1599058800)
            ),
            vec![188, 14, 85, 232, 11, 151, 53]
        );

        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Seperate,
                27,
                20,
                (1597849200, 1599058800)
            ),
            vec![188, 26, 120, 28, 110, 86, 57]
        );

        // case study
        // theta_t = 22 theta_l = 24
        // Time:
        // 大体 17*60 = 1020でハッシュ値が1変わる
        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000,
            latitude: 40.000,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 32]
        );

        let trajectory = Trajectory {
            time: 1597882600,
            longitude: -74.000,
            latitude: 40.000,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 32]
        );

        let trajectory = Trajectory {
            time: 1597882650,
            longitude: -74.000,
            latitude: 40.000,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 33]
        );

        // Latitude:
        // 大体 0.0000165くらいでハッシュ値が1変わる
        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000000,
            latitude: 40.0000049,
        }; // 40.0000049, 40.0000050の間で変わる
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 32]
        );
        // [97, 32]  : 01100001 00100000

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000000,
            latitude: 40.0000050,
        }; // 40.0000049, 40.0000050の間で変わる
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 69, 178]
        );
        // [97, 32]  : 01100001 00100000
        // [69, 178] : 01000101 10110010

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000000,
            latitude: 40.0000214,
        }; // 40.0000214, 40.0000215の間で変わる
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 69, 178]
        );

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000000,
            latitude: 40.0000215,
        }; // 40.0000214, 40.0000215の間で変わる
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 69, 176]
        );
        // [69, 176] : 01000101 10110000

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.000000,
            latitude: 40.0000380,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 69, 162]
        );

        // Longitude:
        // 大体 0.0000215くらいでハッシュ値が1変わる 理論的にも正しい

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.0000009,
            latitude: 40.000,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 32]
        );
        // 32]: 00100001

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.0000010,
            latitude: 40.000,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 4]
        );
        // 4]: 00000100

        let trajectory = Trajectory {
            time: 1597881600,
            longitude: -74.0000225,
            latitude: 40.000,
        }; // .0000224と.0000225の間
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1597881600, 1599090600)
            ),
            vec![1, 164, 83, 226, 16, 194, 97, 0]
        );
        // 4]: 00000000

        // Kinki
        // Lat
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 135.916634,
            latitude: 35.1143095,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1285891200, 1285977540)
            ),
            vec![94, 8, 204, 60, 209, 109, 178]
        );
        // 1143095と1143096の間 1143096
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 135.916634,
            latitude: 35.1143096,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1285891200, 1285977540)
            ),
            vec![94, 8, 204, 60, 209, 109, 176]
        );
        // 1143270と1143271の間 => 0000175
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 135.916634,
            latitude: 35.1143447,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1285891200, 1285977540)
            ),
            vec![94, 8, 204, 60, 209, 109, 160]
        );

        // Long
        // 9166502 と9166503の間
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 135.9166502,
            latitude: 35.1143096,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1285891200, 1285977540)
            ),
            vec![94, 8, 204, 60, 209, 109, 176]
        );
        // 9166717と9166718の間 => 9166718 - 9166503 = 0000215
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 135.9166717,
            latitude: 35.1143096,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1285891200, 1285977540)
            ),
            vec![94, 8, 204, 60, 209, 109, 180]
        );

        // Tokyo
        // Lat
        // 6999128と6999129の間
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 139.774175,
            latitude: 35.6999128,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1222819200, 1222905540)
            ),
            vec![94, 13, 60, 51, 9, 123, 19]
        );
        // 6999302と6999303の間 => 6999303 - 6999129 = 0000174
        let trajectory = Trajectory {
            time: 1285891200,
            longitude: 139.774175,
            latitude: 35.6999302,
        };
        assert_eq!(
            utils::trajectory_hash(
                &trajectory,
                &utils::MixType::Mix,
                22,
                24,
                (1222819200, 1222905540)
            ),
            vec![94, 13, 60, 51, 9, 123, 17]
        );
    }
}
