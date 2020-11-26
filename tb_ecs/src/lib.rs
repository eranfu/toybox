#![feature(type_name_of_val)]

#[macro_use]
extern crate tb_proc_macro;

pub use component::Component;
pub use entity::Entity;
pub use scheduler::Scheduler;
pub use system::data::SystemData;
pub use system::registry::SystemInfo;
pub use system::registry::SystemRegistry;
pub use system::System;
pub use world::World;

mod component;
mod entity;
mod join;
mod scheduler;
mod system;
mod world;
