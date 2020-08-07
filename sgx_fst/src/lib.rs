#![cfg_attr(not(target_env = "sgx"), no_std)]
#[macro_use]
extern crate sgx_tstd as std;

mod lib {
    pub use std::vec::Vec;
    pub use std::cell::RefCell;
    pub use std::slice;
    pub use std::boxed::Box;
    pub use std::string::String;
    pub use std::collections::HashMap;
}


pub use crate::automaton::Automaton;
pub use crate::error::{Error, Result};
pub use crate::map::{Map, MapBuilder};
pub use crate::set::{Set, SetBuilder};
pub use crate::stream::{IntoStreamer, Streamer};

mod bytes;
mod error;
#[path = "automaton/mod.rs"]
mod inner_automaton;
#[path = "map.rs"]
mod inner_map;
#[path = "set.rs"]
mod inner_set;
pub mod raw;
mod stream;

/// Automaton implementations for finite state transducers.
///
/// This module defines a trait, `Automaton`, with several implementations
/// including, but not limited to, union, intersection and complement.
pub mod automaton {
    pub use crate::inner_automaton::*;
}

/// Map operations implemented by finite state transducers.
///
/// This API provided by this sub-module is close in spirit to the API
/// provided by
/// [`std::collections::BTreeMap`](http://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html).
///
/// # Overview of types
///
/// `Map` is a read only interface to pre-constructed sets. `MapBuilder` is
/// used to create new sets. (Once a set is created, it can never be modified.)
/// `Stream`, `Keys` and `Values` are streams that originated from a map.
/// `StreamBuilder` builds range queries. `OpBuilder` collects a set of streams
/// and executes set operations like `union` or `intersection` on them with the
/// option of specifying a merge strategy for a map's values. The rest of the
/// types are streams for set operations.
pub mod map {
    pub use crate::inner_map::*;
}

/// Set operations implemented by finite state transducers.
///
/// This API provided by this sub-module is close in spirit to the API
/// provided by
/// [`std::collections::BTreeSet`](http://doc.rust-lang.org/stable/std/collections/struct.BTreeSet.html).
/// The principle difference, as with everything else in this crate, is that
/// operations are performed on streams of byte strings instead of generic
/// iterators. Another difference is that most of the set operations (union,
/// intersection, difference and symmetric difference) work on multiple sets at
/// the same time, instead of two.
///
/// # Overview of types
///
/// `Set` is a read only interface to pre-constructed sets. `SetBuilder` is
/// used to create new sets. (Once a set is created, it can never be modified.)
/// `Stream` is a stream of values that originated from a set (analogous to an
/// iterator). `StreamBuilder` builds range queries. `OpBuilder` collects a set
/// of streams and executes set operations like `union` or `intersection` on
/// them. The rest of the types are streams for set operations.
pub mod set {
    pub use crate::inner_set::*;
}
