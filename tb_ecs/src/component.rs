use std::marker::PhantomData;

use tb_storage::{Storage, StorageItems};

use crate::entity::Entities;
use crate::system::{ReadAfterWrite, ReadBeforeWrite, ReadOrder};
use crate::world::ResourceId;
use crate::{SystemData, World};

pub trait Component: 'static + Sized {
    type Storage: StorageItems<Data = Self>;
}

pub(crate) struct Components<'r, C, S: 'r> {
    entities: &'r Entities,
    pub(crate) storage: S,
    _phantom: PhantomData<C>,
}

pub struct ReadComponents<'r, C: Component, R: ReadOrder> {
    pub(crate) components: Components<'r, &'r C, &'r Storage<C::Storage>>,
    _phantom: PhantomData<R>,
}

pub struct WriteComponents<'r, C: Component> {
    pub(crate) components: Components<'r, &'r mut C, &'r mut Storage<C::Storage>>,
}

pub type RBWComponents<'r, C> = ReadComponents<'r, C, ReadBeforeWrite>;
pub type RAWComponents<'r, C> = ReadComponents<'r, C, ReadAfterWrite>;

impl<'r, C: Component, R: ReadOrder> ReadComponents<'r, C, R> {
    fn new(world: &'r World) -> Self {
        Self {
            components: Components {
                entities: world.fetch(),
                storage: world.fetch(),
                _phantom: Default::default(),
            },
            _phantom: Default::default(),
        }
    }
}

impl<'r, C: Component> SystemData<'r> for RBWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
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
            components: Components {
                entities: world.fetch(),
                storage: world.fetch_mut(),
                _phantom: Default::default(),
            },
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
        Self::new(world)
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::Storage>(),
        ]
    }
}
