use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;

use tb_core::error::*;

error_chain! {
    errors {
        Fetch
    }
}

type Resources = HashMap<ResourceId, RefCell<Box<dyn Resource>>>;

#[derive(Default)]
pub struct World {
    resources: Resources,
}

impl World {
    pub fn insert<R: Resource>(&mut self, create: impl FnOnce() -> R) -> &mut R {
        let r = self
            .resources
            .entry(ResourceId::new::<R>())
            .or_insert_with(|| RefCell::new(Box::new(create())));
        unsafe { &mut *(r.borrow_mut().as_mut() as *mut dyn Resource as *mut R) }
    }

    pub fn try_fetch<R: Resource>(&self) -> Result<&R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| unsafe { &*(r.borrow().as_ref() as *const dyn Resource as *const R) })
            .chain_err(|| ErrorKind::Fetch)
    }

    pub fn try_fetch_mut<R: Resource>(&self) -> Result<&mut R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| unsafe { &mut *(r.borrow_mut().as_mut() as *mut dyn Resource as *mut R) })
            .chain_err(|| ErrorKind::Fetch)
    }

    pub fn fetch<R: Resource>(&self) -> &R {
        self.try_fetch().unwrap()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn fetch_mut<R: Resource>(&self) -> &mut R {
        self.try_fetch_mut().unwrap()
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

pub trait Resource: 'static {}

impl<R: Any> Resource for R {}

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

    #[test]
    #[should_panic(
        expected = "没有找到 Resource，请先调用 insert 添加 Resource。\nResource type name: [tb_ecs::world::tests::TestResource]"
    )]
    fn fetch_resource_failed() {
        let world = World::default();
        let _test_resource = world.fetch::<TestResource>();
    }

    #[test]
    fn fetch_mut_error() {
        let world = World::default();
        let result = world.try_fetch_mut::<TestResource>();
        assert!(result.is_err());
    }
}
