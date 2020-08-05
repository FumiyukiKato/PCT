/* Constants */

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;
// risk_level 1バイト + qeuryId
pub const QUERY_ID_SIZE_U8: usize = 8;
pub const QUERY_RESULT_U8: usize = 1;
pub const RESPONSE_DATA_SIZE_U8: usize = QUERY_ID_SIZE_U8 + QUERY_RESULT_U8;

pub const THREASHOLD: usize = 100000;

pub const CONTACT_TIME_THREASHOLD: u64 = 600;

// UNIX EPOCH INTERVAL OF THE GPS DATA
pub const TIME_INTERVAL: u64 = 600;

pub const ENCODE_GEOHASH_U8_SIZE: usize = 10;
pub const ENCODE_TIME_U8_SIZE: usize = 4;
pub const ENCODEDVALUE_SIZE: usize = ENCODE_GEOHASH_U8_SIZE + ENCODE_TIME_U8_SIZE;

pub const QUERY_SIZE: usize = 1440;