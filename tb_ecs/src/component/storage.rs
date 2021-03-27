use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::{join, Component, Entity};

pub struct ComponentStorage<T: Component> {
    data: Vec<T>,
    entities: Vec<Entity>,
    entity_to_index: EntityToIndex,
}

impl<T: Component> ComponentStorage<T> {
    pub(crate) fn entity_iter(&self) -> Box<dyn '_ + Iterator<Item = Entity>> {
        Box::new(self.entities.iter().map(|entity| *entity))
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

    pub fn insert(&mut self, entity: Entity, elem: T) {
        match self.entity_to_index.entry(entity) {
            Entry::Occupied(occupied) => self.data[*occupied.get()] = elem,
            Entry::Vacant(vacant) => {
                vacant.insert(self.data.len());
                self.data.push(elem);
                self.entities.push(entity);
            }
        }
    }

    pub fn remove(&mut self, entity: Entity) {
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

#[derive(Default)]
struct EntityToIndex {
    entity_to_index: HashMap<Entity, usize>, // todo: optimize performance
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
