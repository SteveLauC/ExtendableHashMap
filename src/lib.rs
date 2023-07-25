#![feature(vec_push_within_capacity)]
#![feature(extract_if)]

extern crate core;

mod bucket;
mod map;
pub(crate) mod util;

pub use map::HashMap;
