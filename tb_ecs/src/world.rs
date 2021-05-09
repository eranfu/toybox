use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use errors::*;
use tb_core::event_channel::EventChannel;

mod errors {
    pub use tb_core::error::*;

    error_chain! {
        errors {
            Fetch(resource_type_name: String) {
                description("Failed to fetch resource"),
                display("Failed to fetch resource. type_name: {}", resource_type_name),
            }
        }
    }
}

struct ResourceCell(UnsafeCell<Box<dyn Resource>>);

impl ResourceCell {
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut<R: Resource>(&self) -> &mut R {
        let r = &self.0;
        let r = &mut *r.get();
        let r = r.deref_mut();
        &mut *(r as *mut dyn Resource as *mut R)
    }
    pub(crate) unsafe fn get<R: Resource>(&self) -> &R {
        let r = &self.0;
        let r = &*r.get();
        let r = r.deref();
        &*(r as *const dyn Resource as *const R)
    }
}

unsafe impl Sync for ResourceCell {}

type Resources = HashMap<ResourceId, ResourceCell>;

#[derive(Default)]
pub struct World {
    resources: Resources,
    resource_change_events: EventChannel<ResourceChangeEvent>,
}

impl World {
    pub(crate) fn resource_change_events(&self) -> &EventChannel<ResourceChangeEvent> {
        &self.resource_change_events
    }
    pub(crate) fn resource_change_events_mut(&mut self) -> &mut EventChannel<ResourceChangeEvent> {
        &mut self.resource_change_events
    }

    pub fn insert<R: Resource>(&mut self, create: impl FnOnce() -> R) -> &mut R {
        let change_events = &mut self.resource_change_events;
        let res = self
            .resources
            .entry(ResourceId::new::<R>())
            .or_insert_with(|| {
                change_events.push(ResourceChangeEvent::new());
                ResourceCell(UnsafeCell::new(Box::new(create())))
            });

        unsafe { res.get_mut::<R>() }
    }

    /// Fetch immutable resource
    ///
    /// # Safety
    ///
    /// The resource you fetch must meet the reference rules.
    /// If you have got a `mutable resource` by `try_fetch_mut` or `fetch_mut`,
    /// you can no longer fetch a `immutable resource` by this function
    pub unsafe fn try_fetch<R: Resource>(&self) -> errors::Result<&R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| r.get())
            .chain_err(|| errors::ErrorKind::Fetch(std::any::type_name::<R>().into()))
    }

    /// Fetch mutable resource
    ///
    /// # Safety
    ///
    /// The Resource you fetch must meet the reference rules
    /// If you have got a `immutable/mutable resource` by `try_fetch`/`fetch`/`try_fetch_mut`/`fetch_mut`,
    /// you can no longer fetch a `mutable resource` by this function
    pub unsafe fn try_fetch_mut<R: Resource>(&self) -> errors::Result<&mut R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| r.get_mut())
            .chain_err(|| errors::ErrorKind::Fetch(std::any::type_name::<R>().into()))
    }

    /// Fetch immutable resource
    ///
    /// # Safety
    ///
    /// The resource you fetch must meet the reference rules.
    /// If you have got a `mutable resource` by `try_fetch_mut` or `fetch_mut`,
    /// you can no longer fetch a `immutable resource` by this function
    pub unsafe fn fetch<R: Resource>(&self) -> &R {
        self.try_fetch().unwrap()
    }

    /// Fetch mutable resource
    ///
    /// # Safety
    ///
    /// The Resource you fetch must meet the reference rules
    /// If you have got a `immutable/mutable resource` by `try_fetch`/`fetch`/`try_fetch_mut`/`fetch_mut`,
    /// you can no longer fetch a `mutable resource` by this function
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn fetch_mut<R: Resource>(&self) -> &mut R {
        self.try_fetch_mut().unwrap()
    }

    pub fn contains<R: Resource>(&self) -> bool {
        self.contains_id(&ResourceId::new::<R>())
    }

    pub fn contains_id(&self, id: &ResourceId) -> bool {
        self.resources.contains_key(&id)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ResourceId {
    id: TypeId,
}

impl ResourceId {
    pub(crate) fn new<R: Resource + ?Sized>() -> Self {
        ResourceId {
            id: TypeId::of::<R>(),
        }
    }
}

pub trait Resource: 'static + Sync {}

impl<R: 'static + Sync> Resource for R {}

pub struct ResourceChangeEvent {}

impl ResourceChangeEvent {
    fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use crate::World;

    struct TestResource {
        value: i32,
    }

    impl TestResource {
        fn new(value: i32) -> Self {
            Self { value }
        }
    }

    #[test]
    fn world_works() {
        unsafe {
            let mut world = World::default();
            let test_resource = world.try_fetch::<TestResource>();
            assert!(test_resource.is_err());
            world.insert(|| TestResource::new(10));
            assert!(world.try_fetch::<TestResource>().is_ok());
            assert_eq!(world.fetch::<TestResource>().value, 10);
            let resource: &mut TestResource = world.fetch_mut();
            resource.value = 20;
            assert_eq!(world.fetch::<TestResource>().value, 20);
        }
    }

    #[test]
    #[should_panic(expected = "Error(Fetch(\"tb_ecs::world::tests::TestResource\")")]
    fn fetch_resource_failed() {
        let world = World::default();
        let _test_resource = unsafe { world.fetch::<TestResource>() };
    }

    #[test]
    fn fetch_mut_error() {
        let world = World::default();
        let result = unsafe { world.try_fetch_mut::<TestResource>() };
        assert!(result.is_err());
    }
}
