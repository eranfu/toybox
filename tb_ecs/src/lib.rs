#![feature(type_name_of_val)]

pub use component::Component;
pub use entity::Entity;
pub use scheduler::Scheduler;
pub use system::data::SystemData;
pub use system::System;
pub use world::World;

mod component;
mod entity;
mod join;
mod scheduler;
mod system;
mod world;
