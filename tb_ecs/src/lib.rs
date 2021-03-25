#![feature(once_cell)]
#![feature(maybe_uninit_extra)]

pub use inventory;

pub use component::*;
pub use entity::Entities;
pub use entity::Entity;
pub use join::*;
pub use scheduler::Scheduler;
pub use system::data::SystemData;
pub use system::data::Write;
pub use system::data::RAW;
pub use system::data::RBW;
pub use system::registry::SystemInfo;
pub use system::registry::SystemRegistry;
pub use system::System;
pub use tb_ecs_macro::*;
pub use world::Resource;
pub use world::ResourceId;
pub use world::World;

mod component;
mod entity;
mod join;
mod scheduler;
mod system;
mod world;
