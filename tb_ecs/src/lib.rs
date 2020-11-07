#![feature(type_name_of_val)]

pub use scheduler::Scheduler;
pub use system::System;
pub use world::World;

mod entity;
mod component;
mod system;
mod world;
mod scheduler;
