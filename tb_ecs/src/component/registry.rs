use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::marker::PhantomData;
use std::ops::{Deref, Index};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Component, Entity, World};

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ComponentIndex(usize);

impl From<usize> for ComponentIndex {
    fn from(from: usize) -> Self {
        ComponentIndex(from)
    }
}

impl ComponentIndex {
    pub fn get<C: Component>() -> Self {
        let registry = ComponentRegistry::read();
        *registry
            .type_id_to_index
            .get(&ComponentTypeId::new::<C>())
            .unwrap()
    }
}

impl Deref for ComponentIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Index<ComponentIndex> for Vec<&ComponentInfo> {
    type Output = ComponentInfo;

    fn index(&self, index: ComponentIndex) -> &Self::Output {
        self[index.0]
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct ComponentTypeId(TypeId);

impl ComponentTypeId {
    fn new<C: Component>() -> Self {
        Self(TypeId::of::<C>())
    }
}

pub struct ComponentRegistry {
    infos: Vec<&'static ComponentInfo>,
    type_id_to_index: HashMap<ComponentTypeId, ComponentIndex>,
}

impl ComponentRegistry {
    pub fn add_component_infos(component_infos: Box<dyn Iterator<Item = &'static ComponentInfo>>) {
        let cr: &mut ComponentRegistry = &mut Self::write();
        let infos = &mut cr.infos;
        let type_id_to_index = &mut cr.type_id_to_index;
        for info in component_infos {
            match type_id_to_index.entry(info.type_id) {
                Entry::Occupied(occupied) => {
                    let index = unsafe { infos.get_unchecked_mut(**occupied.get()) };
                    *index = info;
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(ComponentIndex(infos.len()));
                    infos.push(info);
                }
            }
        }
    }

    pub fn for_each(op: impl FnMut(&&ComponentInfo)) {
        let this = Self::read();
        let this: &Self = &this;
        this.infos.iter().for_each(op);
    }

    pub(crate) fn operation(
        component_index: ComponentIndex,
    ) -> (
        &'static dyn ComponentOperation,
        RwLockReadGuard<'static, ComponentRegistry>,
    ) {
        let instance = Self::read();
        let operation = &*instance.infos[component_index].operation;
        (unsafe { std::mem::transmute(operation) }, instance)
    }

    fn get_instance() -> &'static RwLock<ComponentRegistry> {
        static INSTANCE: SyncLazy<RwLock<ComponentRegistry>> = SyncLazy::new(|| {
            let mut instance = ComponentRegistry {
                infos: vec![],
                type_id_to_index: Default::default(),
            };

            for info in inventory::iter::<ComponentInfo> {
                instance
                    .type_id_to_index
                    .insert(info.type_id, ComponentIndex(instance.infos.len()));
                instance.infos.push(info);
            }
            RwLock::new(instance)
        });
        &INSTANCE
    }

    fn write() -> RwLockWriteGuard<'static, ComponentRegistry> {
        Self::get_instance().write().unwrap()
    }

    fn read() -> RwLockReadGuard<'static, ComponentRegistry> {
        Self::get_instance().read().unwrap()
    }
}

pub trait ComponentOperation: Send + Sync {
    unsafe fn remove_from_world(&self, world: &World, entity: Entity);
}

struct Operation<C: Component> {
    _phantom: PhantomData<C>,
}

unsafe impl<C: Component> Send for Operation<C> {}

unsafe impl<C: Component> Sync for Operation<C> {}

impl<C: Component> ComponentOperation for Operation<C> {
    unsafe fn remove_from_world(&self, world: &World, entity: Entity) {
        world.fetch_components_mut::<C>().remove(entity)
    }
}

pub struct ComponentInfo {
    type_id: ComponentTypeId,
    operation: Box<dyn ComponentOperation>,
}

impl ComponentInfo {
    pub fn new<C: Component>() -> Self {
        Self {
            type_id: ComponentTypeId::new::<C>(),
            operation: Box::new(Operation::<C> {
                _phantom: Default::default(),
            }),
        }
    }
}

inventory::collect!(ComponentInfo);

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::registry::ComponentIndex;
    use crate::*;

    #[component]
    struct Component0;

    #[component]
    struct Component1;

    #[test]
    fn get_component_index() {
        let mut join_handles = vec![];
        let index_0 = ComponentIndex::get::<Component0>();
        let index_1 = ComponentIndex::get::<Component1>();
        for i in 0..1000 {
            join_handles.push(thread::spawn(move || {
                if i % 2 == 0 {
                    assert_eq!(ComponentIndex::get::<Component0>(), index_0);
                } else {
                    assert_eq!(ComponentIndex::get::<Component1>(), index_1);
                }
            }))
        }
        for join in join_handles {
            join.join().unwrap()
        }
    }
}
