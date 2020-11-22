use std::ops::{Deref, DerefMut};

use crate::component::Component;
use crate::entity::Entities;
use crate::world::{Resource, ResourceId};
use crate::World;

pub trait System<'r> {
    type SystemData: SystemData<'r>;
    fn run(&mut self, system_data: &mut Self::SystemData);
}

pub trait SystemData<'r> {
    fn fetch(world: &'r World) -> Self;
    fn reads_before_write() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
    fn reads_after_write() -> Vec<ResourceId> {
        vec![]
    }
}

/// Read before write
pub struct RBW<'r, R: Resource> {
    resource: &'r R,
}

/// Write
pub struct Write<'r, R: Resource> {
    resource: &'r mut R,
}

/// Read after write
pub struct RAW<'r, R: Resource> {
    resource: &'r R,
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

impl<'r> SystemData<'r> for () {
    fn fetch(_world: &'r World) -> Self {}
}

impl<'r, R: Resource> Deref for RBW<'r, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'r, R: Resource> Deref for Write<'r, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'r, R: Resource> DerefMut for Write<'r, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
    }
}

impl<'r, R: Resource> Deref for RAW<'r, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'r, R: Resource> SystemData<'r> for RBW<'r, R> {
    fn fetch(world: &'r World) -> Self {
        RBW {
            resource: world.fetch(),
        }
    }

    fn reads_before_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
    }
}

impl<'r, R: Resource> SystemData<'r> for Write<'r, R> {
    fn fetch(world: &'r World) -> Self {
        Write {
            resource: world.fetch_mut(),
        }
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
    }
}

impl<'r, R: Resource> SystemData<'r> for RAW<'r, R> {
    fn fetch(world: &'r World) -> Self {
        RAW {
            resource: world.fetch(),
        }
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
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

macro_rules! impl_system_data_tuple {
    ($S0:ident) => {};
    ($S0:ident, $($S1:ident),+) => {
        impl_system_data_tuple!($($S1),+);

        impl<'r, $S0: SystemData<'r>, $($S1: SystemData<'r>),+> SystemData<'r> for ($S0, $($S1),+) {
            fn fetch(world: &'r World) -> Self {
                ($S0::fetch(world), $($S1::fetch(world)),+)
            }

            fn reads_before_write() -> Vec<ResourceId> {
                let mut res = $S0::reads_before_write();
                $({
                    let mut s1_res = $S1::reads_before_write();
                    res.append(&mut s1_res);
                })+
                res
            }

            fn writes() -> Vec<ResourceId> {
                let mut res = $S0::writes();
                $({
                    let mut s1_res = $S1::writes();
                    res.append(&mut s1_res);
                })+
                res
            }

            fn reads_after_write() -> Vec<ResourceId> {
                let mut res = $S0::reads_after_write();
                $({
                    let mut s1_res = $S1::reads_after_write();
                    res.append(&mut s1_res);
                })+
                res
            }
        }
    }
}

impl_system_data_tuple!(S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15);

#[cfg(test)]
mod tests {
    use rayon::ThreadPoolBuilder;

    use crate::{Scheduler, System, World};

    struct TestSystem {}

    impl<'r> System<'r> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: &mut Self::SystemData) {}
    }

    #[test]
    fn empty_system_data() {
        let mut thread_pool = ThreadPoolBuilder::new().build().unwrap();
        let mut scheduler = Scheduler::new(&mut thread_pool);
        scheduler.insert(TestSystem {});
        let world = World::default();
        scheduler.schedule(&world);
    }
}
