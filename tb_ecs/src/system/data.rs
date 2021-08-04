use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub(crate) use crate::system::data::access_order::AccessOrder;
use crate::world::{Resource, ResourceId};
use crate::World;

pub trait SystemData<'r> {
    /// Fetch SystemData
    ///
    /// # Safety
    ///
    /// The SystemData must meet the reference rules.
    unsafe fn fetch(world: &'r World) -> Self;

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

pub struct ResourceAccessor<R, A: AccessOrder> {
    resource: R,
    _phantom: PhantomData<A>,
}

pub(crate) mod access_order {
    pub struct ReadBeforeWrite;

    pub struct Write;

    pub struct ReadAfterWrite;

    pub trait AccessOrder: Sync {}

    impl AccessOrder for ReadBeforeWrite {}

    impl AccessOrder for Write {}

    impl AccessOrder for ReadAfterWrite {}
}

#[allow(type_alias_bounds)]
pub type Read<'r, R: Resource, A> = ResourceAccessor<&'r R, A>;
pub type RBW<'r, R> = Read<'r, R, access_order::ReadBeforeWrite>;
pub type RAW<'r, R> = Read<'r, R, access_order::ReadAfterWrite>;

#[allow(type_alias_bounds)]
pub type Write<'r, R: Resource> = ResourceAccessor<&'r mut R, access_order::Write>;

impl<'r> SystemData<'r> for () {
    unsafe fn fetch(_world: &'r World) -> Self {}
}

impl<'r, R, A: AccessOrder> Deref for Read<'r, R, A> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'r, R> Deref for Write<'r, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'r, R> DerefMut for Write<'r, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
    }
}

impl<'r, R: Resource> SystemData<'r> for RBW<'r, R> {
    unsafe fn fetch(world: &'r World) -> Self {
        RBW {
            resource: world.fetch(),
            _phantom: Default::default(),
        }
    }

    fn reads_before_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
    }
}

impl<'r, R: Resource> SystemData<'r> for Write<'r, R> {
    unsafe fn fetch(world: &'r World) -> Self {
        Write {
            resource: world.fetch_mut(),
            _phantom: Default::default(),
        }
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
    }
}

impl<'r, R: Resource> SystemData<'r> for RAW<'r, R> {
    unsafe fn fetch(world: &'r World) -> Self {
        RAW {
            resource: world.fetch(),
            _phantom: Default::default(),
        }
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<R>()]
    }
}

macro_rules! impl_system_data_tuple {
    ($S0:ident) => {};
    ($S0:ident, $($S1:ident),+) => {
        impl_system_data_tuple!($($S1),+);

        impl<'r, $S0: SystemData<'r>, $($S1: SystemData<'r>),+> SystemData<'r> for ($S0, $($S1),+) {
            unsafe fn fetch(world: &'r World) -> Self {
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
