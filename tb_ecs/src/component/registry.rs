use std::slice::SliceIndex;

use crate::{ComponentIndex, Entity, World};

pub struct ComponentRegistry {
    infos: Vec<ComponentInfo>,
}

pub trait ComponentOperation {
    fn remove_from_world(&self, world: &mut World, entity: Entity);
}

pub struct ComponentInfo {
    operation: Box<dyn ComponentOperation>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ComponentIndex(usize);
