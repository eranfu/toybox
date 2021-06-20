#![feature(maybe_uninit_extra)]
#![feature(allocator_api)]
#![feature(layout_for_ptr)]

pub use nalgebra as math;

pub mod algorithm;
pub mod collections;
pub mod error;
pub mod event_channel;
pub mod path_util;

pub mod serde {
    pub use serde::*;
    pub use serde_box::*;
    pub use serde_json;
}
