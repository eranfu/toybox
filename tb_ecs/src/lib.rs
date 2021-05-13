#![feature(once_cell)]
#![feature(maybe_uninit_extra)]

pub use inventory;
pub use serde::{Deserialize, Serialize};

pub use component::*;
pub use entity::*;
pub use join::*;
pub use scheduler::*;
pub use system::*;
pub use tb_ecs_macro::*;
pub use world::*;

mod component;
mod entity;
mod join;
mod scheduler;
mod system;
mod world;
