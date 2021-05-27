use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::slice::Iter;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_box::*;

use crate::{
    join, Component, ComponentWithEntityRef, Entities, Entity, EntityRef, SystemData, World,
    WriteComponents,
};

#[serde_box]
pub trait ComponentStorageTrait: Send + Sync + SerdeBoxSer + SerdeBoxDe {
    fn attach_to_world(&self, world: &mut World, link: &mut LocalToWorldLink);
}

#[derive(Serialize, Deserialize)]
pub struct ComponentStorage<C: Component> {
    data: Vec<C>,
    entities: Vec<Entity>,
    entity_to_index: EntityToIndex,
}

impl<T: Component> ComponentStorage<T> {
    pub fn open(&self) -> (Iter<'_, Entity>, &ComponentStorage<T>) {
        (self.entities.iter(), self)
    }

    pub(crate) fn entity_iter(&self) -> Box<dyn '_ + Iterator<Item = Entity>> {
        Box::new(self.entities.iter().copied())
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains(entity)
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub(crate) fn insert(&mut self, entity: Entity, elem: T) {
        match self.entity_to_index.entry(entity) {
            Entry::Occupied(occupied) => self.data[*occupied.get()] = elem,
            Entry::Vacant(vacant) => {
                vacant.insert(self.data.len());
                self.data.push(elem);
                self.entities.push(entity);
            }
        }
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        if let Some(removed_index) = self.entity_to_index.remove(&entity) {
            let last_entity = *self.entities.last().unwrap();
            self.entities.swap_remove(removed_index);
            self.data.swap_remove(removed_index);
            if last_entity != entity {
                self.entity_to_index.insert(last_entity, removed_index);
            }
        }
    }
}

impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            entities: Default::default(),
            entity_to_index: Default::default(),
        }
    }
}

#[serde_box]
impl<C: Component + Serialize + DeserializeOwned> ComponentStorageTrait for ComponentStorage<C> {
    default fn attach_to_world(&self, world: &mut World, link: &mut LocalToWorldLink) {
        world.insert_components::<C>();
        world.insert(Entities::default);
        let mut components_in_world = unsafe { WriteComponents::<C>::fetch(world) };
        let entities = unsafe { world.fetch::<Entities>() };
        let (entity, components) = (self.entities.iter(), self.data.iter());
        entity.zip(components).for_each(|(&entity, component)| {
            components_in_world.insert(link.build_link(entity, entities), component.clone());
        });
    }
}

#[serde_box]
impl<C: Serialize + DeserializeOwned> ComponentStorageTrait for ComponentStorage<C>
where
    for<'e> C: ComponentWithEntityRef<'e>,
{
    fn attach_to_world(&self, world: &mut World, link: &mut LocalToWorldLink) {
        world.insert_components::<C>();
        let mut components_in_world = unsafe { WriteComponents::<C>::fetch(world) };
        let entities = unsafe { world.fetch_mut::<Entities>() };
        let (entity, components) = (self.entities.iter(), self.data.iter());
        entity.zip(components).for_each(|(&entity, component)| {
            let mut component: C = component.clone();
            let mut entity_ref = component.get_entity_ref();
            ConvertToWorld::convert_to_world(&mut entity_ref, link, entities);
            drop(entity_ref);
            components_in_world.insert(link.build_link(entity, entities), component);
        });
    }
}

impl<'s, T: Component> join::ElementFetcher for &'s ComponentStorage<T> {
    type Element = &'s T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        self.entity_to_index
            .get(&entity)
            .map(|index| &self.data[*index])
    }
}

impl<'s, T: Component> join::ElementFetcher for &'s mut ComponentStorage<T> {
    type Element = &'s mut T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        let s: &'s mut Self = unsafe { std::mem::transmute(self) };
        let data = &mut s.data;
        s.entity_to_index
            .get(&entity)
            .map(move |&index| &mut data[index])
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct LocalToWorldLink(HashMap<Entity, Entity>);

impl LocalToWorldLink {
    pub fn build_link(&mut self, local: Entity, entities: &Entities) -> Entity {
        match self.0.get(&local) {
            None => {
                let entity = entities.new_entity();
                self.0.insert(local, entity);
                entity
            }
            Some(entity) => *entity,
        }
    }
}

pub trait ConvertToWorld {
    fn convert_to_world(&mut self, link: &mut LocalToWorldLink, entities: &mut Entities);
}

impl<E: EntityRef> ConvertToWorld for E {
    default fn convert_to_world(&mut self, link: &mut LocalToWorldLink, entities: &mut Entities) {
        self.for_each(&mut |e: &mut Entity| {
            *e = link.build_link(*e, entities);
        });
    }
}

#[derive(Default, Serialize, Deserialize)]
struct EntityToIndex {
    entity_to_index: HashMap<Entity, usize>,
}

impl EntityToIndex {
    pub(crate) fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }
    pub(crate) fn get(&self, entity: &Entity) -> Option<&usize> {
        self.entity_to_index.get(entity)
    }
    pub(crate) fn insert(&mut self, entity: Entity, index: usize) -> Option<usize> {
        self.entity_to_index.insert(entity, index)
    }
    pub(crate) fn remove(&mut self, entity: &Entity) -> Option<usize> {
        self.entity_to_index.remove(entity)
    }
    pub(crate) fn entry(&mut self, entity: Entity) -> Entry<'_, Entity, usize> {
        self.entity_to_index.entry(entity)
    }
}
