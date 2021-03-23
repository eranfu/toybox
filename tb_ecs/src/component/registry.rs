use crate::{Entity, World};

pub struct ComponentRegistry {
    infos: Vec<ComponentInfo>,
}

pub trait ComponentOperation {
    fn remove_from_world(&self, world: &mut World, entity: Entity);
}

pub struct ComponentInfo {
    operation: Box<dyn ComponentOperation>,
}

inventory::collect!(ComponentInfo);
