use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::{join, Component, Entities, Entity, EntityRef};

#[derive(Serialize, Deserialize)]
pub struct ComponentStorage<C: Component> {
    components: Vec<C>,
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
        self.components.len()
    }
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn insert(&mut self, entity: Entity, elem: T) {
        match self.entity_to_index.entry(entity) {
            Entry::Occupied(occupied) => self.components[*occupied.get()] = elem,
            Entry::Vacant(vacant) => {
                vacant.insert(self.components.len());
                self.components.push(elem);
                self.entities.push(entity);
            }
        }
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        if let Some(removed_index) = self.entity_to_index.remove(&entity) {
            let last_entity = *self.entities.last().unwrap();
            self.entities.swap_remove(removed_index);
            self.components.swap_remove(removed_index);
            if last_entity != entity {
                self.entity_to_index.insert(last_entity, removed_index);
            }
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.entity_to_index
            .get(&entity)
            .map(|&index| &self.components[index])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        match self.entity_to_index.get(&entity) {
            None => None,
            Some(&index) => Some(&mut self.components[index]),
        }
    }
}

impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self {
            components: Default::default(),
            entities: Default::default(),
            entity_to_index: Default::default(),
        }
    }
}

impl<'s, T: Component> join::ElementFetcher for &'s ComponentStorage<T> {
    type Element = &'s T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        self.get(entity)
    }
}

impl<'s, T: Component> join::ElementFetcher for &'s mut ComponentStorage<T> {
    type Element = &'s mut T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        let s: &'s mut Self = unsafe { std::mem::transmute(self) };
        s.get_mut(entity)
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
    fn convert_to_world(&mut self, link: &mut LocalToWorldLink, entities: &mut Entities) {
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
