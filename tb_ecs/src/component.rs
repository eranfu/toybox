use tb_storage::Storage;

use crate::entity::Entities;
use crate::world::ResourceId;
use crate::{SystemData, World};

pub trait Component: 'static + Sized {
    type Storage: Storage<Data = Self>;
}

pub struct RBWComponents<'r, C: Component> {
    entities: &'r Entities,
    components: &'r C::Storage,
}

pub struct WriteComponents<'r, C: Component> {
    entities: &'r Entities,
    components: &'r mut C::Storage,
}

pub struct RAWComponents<'r, C: Component> {
    entities: &'r Entities,
    components: &'r C::Storage,
}

impl<'r, C: Component> RBWComponents<'r, C> {
    pub(crate) fn components(&'r self) -> &'r C::Storage {
        &self.components
    }
    pub(crate) fn entities(&'r self) -> &'r Entities {
        &self.entities
    }
}

impl<'r, C: Component> WriteComponents<'r, C> {
    pub(crate) fn components(&'r self) -> &'r C::Storage {
        &self.components
    }
    fn entities(&'r self) -> &'r Entities {
        &self.entities
    }
}

impl<'r, C: Component> RAWComponents<'r, C> {
    pub(crate) fn components(&'r self) -> &'r C::Storage {
        &self.components
    }
    fn entities(&'r self) -> &'r Entities {
        &self.entities
    }
}

impl<'r, C: Component> SystemData<'r> for RBWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            components: world.fetch(),
        }
    }

    fn reads_before_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::Storage>(),
        ]
    }
}

impl<'r, C: Component> SystemData<'r> for WriteComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            components: world.fetch_mut(),
        }
    }

    fn writes() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::Storage>(),
        ]
    }
}

impl<'r, C: Component> SystemData<'r> for RAWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            components: world.fetch(),
        }
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::Storage>(),
        ]
    }
}
