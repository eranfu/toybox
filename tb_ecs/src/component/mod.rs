use std::marker::PhantomData;
use std::ops::Not;

pub use anti_components::*;
pub use registry::*;
pub use storage::*;
pub use tb_core::*;

use crate::*;

mod anti_components;
pub(crate) mod registry;
mod storage;

#[serde_box]
pub trait Component: 'static + Send + Sync + SerdeBoxSer + SerdeBoxDe {}

pub trait EntityRef {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity));
}

pub trait ComponentWithEntityRef<'e>: Component {
    type Ref: 'e + EntityRef;
    fn mut_entity_ref(&'e mut self) -> Self::Ref;
}

pub struct Components<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    entities: &'r Entities,
    storage: S,
    _phantom: PhantomData<(C, A)>,
}

pub trait Storage: Sync {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains(&self, entity: Entity) -> bool;
}

pub type ReadComps<'r, C, A> = Components<'r, &'r ComponentStorage<C>, C, A>;
pub type RBWComps<'r, C> = ReadComps<'r, C, access_order::ReadBeforeWrite>;
pub type RAWComps<'r, C> = ReadComps<'r, C, access_order::ReadAfterWrite>;
pub type WriteComps<'r, C> = Components<'r, &'r mut ComponentStorage<C>, C, access_order::Write>;

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

impl<'e> EntityRef for &'e mut Vec<Entity> {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity)) {
        self.iter_mut().for_each(action)
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

impl<'r, C: Component, A: AccessOrder> Not for &'r ReadComps<'r, C, A> {
    type Output = anti_components::AntiComponents<'r, &'r ComponentStorage<C>, C, A>;

    fn not(self) -> Self::Output {
        anti_components::AntiComponents::new(self)
    }
}

impl<'r, C: Component> Not for &'r mut WriteComps<'r, C> {
    type Output =
        anti_components::AntiComponents<'r, &'r mut ComponentStorage<C>, C, access_order::Write>;

    fn not(self) -> Self::Output {
        anti_components::AntiComponents::new(self)
    }
}

impl<'r, C: Component, A: AccessOrder> Join<'r> for &'r ReadComps<'r, C, A> {
    type Element = C;
    type ElementFetcher = &'r ComponentStorage<C>;

    fn open(
        mut self,
    ) -> (
        Box<dyn 'r + Iterator<Item = Entity> + Send>,
        Self::ElementFetcher,
    ) {
        (self.storage.entity_iter(), self.elem_fetcher())
    }

    fn entities(&self) -> &'r Entities {
        self.entities
    }

    fn len(&self) -> usize {
        self.storage.len()
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        self.storage
    }

    fn get_matched_entities(&self) -> Box<dyn 'r + Iterator<Item = Entity> + Send> {
        Box::new(self.storage.entity_iter())
    }

    fn fill_matcher(matcher: &mut ArchetypeMatcher) {
        matcher.add_all(ComponentIndex::get::<C>());
    }
}

impl<'r, C: Component> Join<'r> for &'r mut WriteComps<'r, C> {
    type Element = C;
    type ElementFetcher = &'r mut ComponentStorage<C>;

    fn open(
        self,
    ) -> (
        Box<dyn 'r + Iterator<Item = Entity> + Send>,
        Self::ElementFetcher,
    ) {
        let storage = unsafe { &mut *(&mut self.storage as *mut _ as *mut _) };
        (self.get_matched_entities(), storage)
    }

    fn entities(&self) -> &'r Entities {
        self.entities
    }

    fn len(&self) -> usize {
        self.storage.len()
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        let s: &'r mut Self = unsafe { std::mem::transmute(self) };
        s.storage
    }

    fn get_matched_entities(&self) -> Box<dyn 'r + Iterator<Item = Entity> + Send> {
        let components: &'r WriteComps<'r, C> = unsafe { &*(self as *const &mut _ as *const _) };
        components.storage.entity_iter()
    }

    fn fill_matcher(matcher: &mut ArchetypeMatcher) {
        matcher.add_all(ComponentIndex::get::<C>())
    }
}

impl<'r, C: Component, A: AccessOrder> ReadComps<'r, C, A> {
    unsafe fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_components::<C>(),
            _phantom: Default::default(),
        }
    }
}

impl<'r, C: Component> WriteComps<'r, C> {
    unsafe fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_components_mut::<C>(),
            _phantom: Default::default(),
        }
    }
    pub fn insert(&mut self, entity: Entity, component: C) {
        self.storage.insert(entity, component);
        self.entities.on_component_inserted::<C>(entity);
    }
}

impl<'r, C: Component> SystemData<'r> for RBWComps<'r, C> {
    unsafe fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn reads_before_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<ComponentStorage<C>>(),
        ]
    }
}

impl<'r, C: Component> SystemData<'r> for WriteComps<'r, C> {
    unsafe fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<ComponentStorage<C>>()]
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<Entities>()]
    }
}

impl<'r, C: Component> SystemData<'r> for RAWComps<'r, C> {
    unsafe fn fetch(world: &'r World) -> Self {
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
    /// Fetch immutable components
    ///
    /// # Safety
    ///
    /// see `World::fetch`
    pub unsafe fn fetch_components<C: Component>(&self) -> &ComponentStorage<C> {
        self.fetch()
    }

    /// Fetch mutable components
    ///
    /// # Safety
    ///
    /// see `World::fetch_mut`
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn fetch_components_mut<C: Component>(&self) -> &mut ComponentStorage<C> {
        self.fetch_mut()
    }

    pub fn insert_components<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.insert(ComponentStorage::<C>::default)
    }
}

#[cfg(test)]
mod tests {
    use tb_core::*;
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
        let components1 = unsafe { RAWComps::<Component1>::fetch(&world) };
        let mut components2 = unsafe { WriteComps::<Component2>::fetch(&world) };
        for _x in (&components1, &mut components2).join() {
            unreachable!()
        }

        let _entity = world
            .create_entity()
            .with(Component1 { value1: 1 })
            .with(Component2 { value2: 2 })
            .create();
        let components1 = unsafe { RAWComps::<Component1>::fetch(&world) };
        let mut components2 = unsafe { WriteComps::<Component2>::fetch(&world) };
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
            unsafe { <(RBWComps<Component1>, RBWComps<Component2>)>::fetch(&world) };
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
        let mut components1 = unsafe { WriteComps::<Component1>::fetch(&world) };
        for component1 in (&mut components1).join() {
            assert_eq!(component1.value1, 10);
        }
    }
}
