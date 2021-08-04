#![feature(once_cell)]
#![feature(maybe_uninit_extra)]
#![feature(type_alias_impl_trait)]

pub use inventory;

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
