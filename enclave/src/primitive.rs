use std::string::String;
use constant::*;

pub type UnixEpoch = u64;
pub fn unixepoch_from_u8(u_timestamp: [u8; UNIXEPOCH_U8_SIZE]) -> UnixEpoch {
    let s_timestamp = String::from_utf8(u_timestamp.to_vec()).unwrap();
    let num: UnixEpoch = (&s_timestamp).parse().unwrap();
    num
}

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

pub type EncodedValue = [u8; ENCODEDVALUE_SIZE];