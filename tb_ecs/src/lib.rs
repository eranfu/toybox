#![feature(type_name_of_val)]

pub use component::Component;
pub use entity::Entity;
pub use scheduler::Scheduler;
pub use system::System;
pub use system::SystemData;
pub use world::World;

mod entity;
mod component;
mod system;
mod world;
mod scheduler;
