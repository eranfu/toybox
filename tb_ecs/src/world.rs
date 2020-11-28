use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use tb_core::AnyErrorResult;

type Resources = HashMap<ResourceId, RefCell<Box<dyn Resource>>>;

#[derive(Default)]
pub struct World {
    resources: Resources,
}

struct FetchError<R: Resource> {
    _phantom: PhantomData<R>,
}

#[derive(Eq, PartialEq, Hash)]
pub struct ResourceId {
    id: TypeId,
}

pub trait Resource: 'static {}

impl<R: Any> Resource for R {}

impl<R: Resource> Default for FetchError<R> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<R: Resource> Debug for FetchError<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

impl<R: Resource> Display for FetchError<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "没有找到 Resource，请先调用 insert 添加 Resource。\nResource type name: [{}]",
            std::any::type_name::<R>()
        )
    }
}

impl<R: Resource> Error for FetchError<R> {}

impl World {
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let resource_id = ResourceId::new::<R>();
        assert!(!self.resources.contains_key(&resource_id));
        self.resources
            .insert(resource_id, RefCell::new(Box::new(resource)));
    }

    pub fn entry<R: Resource>(&mut self) -> Entry<ResourceId, RefCell<Box<dyn Resource>>> {
        self.resources.entry(ResourceId::new::<R>())
    }

    pub fn try_fetch<R: Resource>(&self) -> AnyErrorResult<&R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| unsafe { &*(r.borrow().as_ref() as *const dyn Resource as *const R) })
            .ok_or_else(|| FetchError::<R>::default().into())
    }

    pub fn try_fetch_mut<R: Resource>(&self) -> AnyErrorResult<&mut R> {
        self.resources
            .get(&ResourceId::new::<R>())
            .map(|r| unsafe { &mut *(r.borrow_mut().as_mut() as *mut dyn Resource as *mut R) })
            .ok_or_else(|| FetchError::<R>::default().into())
    }

    pub fn fetch_or_insert_default<R: Resource + Default>(&mut self) -> &mut R {
        let r = self
            .entry::<R>()
            .or_insert_with(|| RefCell::new(Box::new(<R as Default>::default())));
        unsafe { &mut *((*r).borrow_mut().as_mut() as *mut dyn Resource as *mut R) }
    }

    pub fn fetch<R: Resource>(&self) -> &R {
        self.try_fetch().unwrap()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn fetch_mut<R: Resource>(&self) -> &mut R {
        self.try_fetch_mut().unwrap()
    }
}

impl ResourceId {
    pub(crate) fn new<R: Resource + ?Sized>() -> Self {
        ResourceId {
            id: TypeId::of::<R>(),
        }
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
        let mut world = World::default();
        let test_resource = world.try_fetch::<TestResource>();
        assert!(test_resource.is_err());
        world.insert(TestResource::new(10));
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
