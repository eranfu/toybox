use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

use crate::component::storage::ComponentStorage;
use crate::entity::Entities;
use crate::join::Join;
use crate::system::data::{access_order, AccessOrder};
use crate::world::ResourceId;
use crate::{Entity, SystemData, World};

mod anti_components;
mod storage;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ComponentIndex(usize);

pub trait Component: 'static + Sized + Clone {}

pub trait EntityRef {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity));
}

pub trait ComponentWithEntityRef<'e>: Component {
    type Ref: 'e + EntityRef;
    fn get_entity_ref(&'e mut self) -> Self::Ref;
}

pub struct Components<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    entities: &'r Entities,
    storage: S,
    _phantom: PhantomData<(C, A)>,
}

pub trait Storage {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains(&self, entity: Entity) -> bool;
}

pub type ReadComponents<'r, C, A> = Components<'r, &'r ComponentStorage<C>, C, A>;
pub type RBWComponents<'r, C> = ReadComponents<'r, C, access_order::ReadBeforeWrite>;
pub type RAWComponents<'r, C> = ReadComponents<'r, C, access_order::ReadAfterWrite>;
pub type WriteComponents<'r, C> =
    Components<'r, &'r mut ComponentStorage<C>, C, access_order::Write>;

impl<'r, C: Component> Storage for &'r ComponentStorage<C> {
    fn len(&self) -> usize {
        ComponentStorage::len(self)
    }

    fn contains(&self, entity: Entity) -> bool {
        ComponentStorage::contains(self, entity)
    }
}

impl<'r, C: Component> Storage for &'r mut ComponentStorage<C> {
    fn len(&self) -> usize {
        ComponentStorage::len(self)
    }

    fn contains(&self, entity: Entity) -> bool {
        ComponentStorage::contains(self, entity)
    }
}

impl<'e> EntityRef for &'e mut Entity {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity)) {
        action(self)
    }
}

macro_rules! impl_entity_ref_tuple {
    ($e:ident) => {};
    ($e0:ident, $($e1:ident), +) => {
        impl_entity_ref_tuple!($($e1), +);

        impl<'e, $e0: EntityRef, $($e1: EntityRef), +> EntityRef for ($e0, $($e1), +) {
            #[allow(non_snake_case)]
            fn for_each(&mut self, action: &mut impl FnMut(&mut Entity)) {
                let ($e0, $($e1), +) = self;
                $e0.for_each(action);
                $($e1.for_each(action));
                +;
            }
        }
    };
}

impl_entity_ref_tuple!(E0, E1, E2, E3, E4, E5, E6, E7);

impl<'r, C: Component, A: AccessOrder> Not for &'r ReadComponents<'r, C, A> {
    type Output = anti_components::AntiComponents<'r, &'r ComponentStorage<C>, C, A>;

    fn not(self) -> Self::Output {
        anti_components::AntiComponents::new(self)
    }
}

impl<'r, C: Component> Not for &'r mut WriteComponents<'r, C> {
    type Output =
        anti_components::AntiComponents<'r, &'r mut ComponentStorage<C>, C, access_order::Write>;

    fn not(self) -> Self::Output {
        anti_components::AntiComponents::new(self)
    }
}

impl<'r, C: Component, A: AccessOrder> Join<'r> for &'r ReadComponents<'r, C, A> {
    type ElementFetcher = &'r ComponentStorage<C>;

    fn len(&self) -> usize {
        self.storage.len()
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        self.storage
    }
}

impl<'r, C: Component> Join<'r> for &'r mut WriteComponents<'r, C> {
    type ElementFetcher = &'r mut ComponentStorage<C>;

    fn len(&self) -> usize {
        self.storage.len()
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        let s: &'r mut Self = unsafe { std::mem::transmute(self) };
        s.storage
    }
}

impl<'r, C: Component, A: AccessOrder> Deref for ReadComponents<'r, C, A> {
    type Target = ComponentStorage<C>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<'r, C: Component> Deref for WriteComponents<'r, C> {
    type Target = ComponentStorage<C>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<'r, C: Component> DerefMut for WriteComponents<'r, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

impl<'r, C: Component, A: AccessOrder> ReadComponents<'r, C, A> {
    fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_components::<C>(),
            _phantom: Default::default(),
        }
    }
}

impl<'r, C: Component> WriteComponents<'r, C> {
    fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_components_mut::<C>(),
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
            ResourceId::new::<ComponentStorage<C>>(),
        ]
    }
}

impl<'r, C: Component> SystemData<'r> for WriteComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<ComponentStorage<C>>()]
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<Entities>()]
    }
}

impl<'r, C: Component> SystemData<'r> for RAWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<ComponentStorage<C>>(),
        ]
    }
}

impl World {
    pub fn fetch_components<C: Component>(&self) -> &ComponentStorage<C> {
        self.fetch()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn fetch_components_mut<C: Component>(&self) -> &mut ComponentStorage<C> {
        self.fetch_mut()
    }

    pub fn insert_components<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.insert(ComponentStorage::<C>::default)
    }
}

#[cfg(test)]
mod tests {
    use tb_ecs_macro::*;

    use crate::component::storage::ComponentStorage;
    use crate::*;

    #[component]
    struct Component1 {
        value1: i32,
    }

    #[component]
    struct Component2 {
        value2: i32,
    }

    #[test]
    fn it_works() {
        let mut world = World::default();
        world.insert(Entities::default);
        world.insert(ComponentStorage::<Component1>::default);
        world.insert(ComponentStorage::<Component2>::default);
        let components1 = RAWComponents::<Component1>::fetch(&world);
        let mut components2 = WriteComponents::<Component2>::fetch(&world);
        for _x in (&components1, &mut components2).join() {
            unreachable!()
        }

        let _entity = world
            .create_entity()
            .with(Component1 { value1: 1 })
            .with(Component2 { value2: 2 })
            .create();
        let components1 = RAWComponents::<Component1>::fetch(&world);
        let mut components2 = WriteComponents::<Component2>::fetch(&world);
        let (v1, v2): (&Component1, &mut Component2) =
            (&components1, &mut components2).join().next().unwrap();
        assert_eq!(v1.value1, 1);
        assert_eq!(v2.value2, 2);
    }

    #[test]
    fn anti_components() {
        let mut world = World::default();
        world
            .create_entity()
            .with(Component1 { value1: 1 })
            .with(Component2 { value2: 2 })
            .create();
        world
            .create_entity()
            .with(Component1 { value1: 11 })
            .create();

        let (components1, components2) =
            <(RBWComponents<Component1>, RBWComponents<Component2>)>::fetch(&world);
        let mut has = false;
        for (component1, component2) in (&components1, &components2).join() {
            has = true;
            assert_eq!(component1.value1, 1);
            assert_eq!(component2.value2, 2);
        }
        assert!(has);

        let mut has = false;
        for (component1, _) in (&components1, !&components2).join() {
            has = true;
            assert_eq!(component1.value1, 11);
        }
        assert!(has);

        for (_, _) in (!&components1, &components2).join() {
            unreachable!()
        }
    }

    #[test]
    fn write_components_open() {
        let mut world = World::default();
        world
            .create_entity()
            .with(Component1 { value1: 10 })
            .create();
        let mut components1 = WriteComponents::<Component1>::fetch(&world);
        for component1 in (&mut components1).join() {
            assert_eq!(component1.value1, 10);
        }
    }
}
