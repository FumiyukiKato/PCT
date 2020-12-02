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

#[cfg(feature = "gp10")]
pub const ENCODEDVALUE_SIZE: usize = 14;

#[cfg(feature = "th72")]
pub const ENCODEDVALUE_SIZE: usize = 9;

#[cfg(feature = "th48")]
pub const ENCODEDVALUE_SIZE: usize = 6;

#[cfg(feature = "th54")]
pub const ENCODEDVALUE_SIZE: usize = 7;

#[cfg(feature = "th60")]
pub const ENCODEDVALUE_SIZE: usize = 8;

pub const QUERY_SIZE: usize = 1440;

// for optimization
pub const CLIENT_SIZE: usize = 4500;


// for secure channel encryption
pub const COUNTER_BLOCK: [u8; 16] = [0; 16];
pub const SGXSSL_CTR_BITS: u32 = 128;
pub const QUERY_BYTES: usize = QUERY_SIZE*ENCODEDVALUE_SIZE;