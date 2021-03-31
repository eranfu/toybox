#![feature(backtrace)]

pub use ::error_chain::error_chain;

pub use error::AnyErrorResult;

pub mod algorithm;
pub mod error;
