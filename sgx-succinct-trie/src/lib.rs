#![feature(core_intrinsics)]
#![warn(non_camel_case_types)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#[macro_use]
extern crate sgx_tstd as std;

mod lib {
    pub use std::vec::Vec;
    pub use std::slice;
}

mod bitvector;
mod builder;
pub mod config;
mod label_vector;
mod louds_dense;
mod louds_sparse;
mod popcount;
mod rank;
mod select;
mod suffix;
mod cache;
pub mod trie;
