use std::collections::HashMap;

use crate::join::ElementFetcher;
use crate::{join, Entity};

pub struct SparseSet<T> {
    data: Vec<T>,
    entities: Vec<Entity>,
    entity_to_index: EntityToIndex,
}

impl<T> SparseSet<T> {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn open(&self) -> (SparseSetEntityIter, SparseSetFetch<T>) {
        (self.iter(), self.fetch_elem())
    }

    pub fn open_mut(&mut self) -> (SparseSetEntityIter, SparseSetFetchMut<T>) {
        (
            SparseSetEntityIter {
                entities_iter: self.entities.iter(),
            },
            SparseSetFetchMut {
                entity_to_index: &self.entity_to_index,
                data: &mut self.data,
            },
        )
    }

    pub(crate) fn iter(&self) -> SparseSetEntityIter {
        SparseSetEntityIter {
            entities_iter: self.entities.iter(),
        }
    }

    pub fn fetch_elem(&self) -> SparseSetFetch<T> {
        SparseSetFetch {
            entity_to_index: &self.entity_to_index,
            data: &self.data,
        }
    }

    pub fn fetch_elem_mut(&mut self) -> SparseSetFetchMut<T> {
        SparseSetFetchMut {
            entity_to_index: &self.entity_to_index,
            data: &mut self.data,
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.fetch_elem().contains(entity)
    }

    pub fn insert(&mut self, entity: Entity, elem: T) {
        let index = self.entities.len();
        match self.entity_to_index.insert(entity, index) {
            None => {
                self.entities.push(entity);
                self.data.push(elem);
            }
            Some(old_index) => {
                self.data[old_index] = elem;
            }
        }
    }

    pub fn remove(&mut self, entity: Entity) {
        match self.entity_to_index.remove(entity) {
            None => {}
            Some(remove_index) => {
                let last_entity = *self.entities.last().unwrap();
                self.entities.swap_remove(remove_index);
                self.data.swap_remove(remove_index);
                if last_entity != entity {
                    self.entity_to_index.insert(last_entity, remove_index);
                }
            }
        }
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            entities: Default::default(),
            entity_to_index: Default::default(),
        }
    }
}

impl<'s, T: 's> IntoIterator for &'s SparseSet<T> {
    type Item = Entity;
    type IntoIter = SparseSetEntityIter<'s>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct SparseSetEntityIter<'s> {
    entities_iter: std::slice::Iter<'s, Entity>,
}

impl<'s> Iterator for SparseSetEntityIter<'s> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities_iter.next().copied()
    }
}

pub struct SparseSetFetch<'s, T> {
    entity_to_index: &'s EntityToIndex,
    data: &'s Vec<T>,
}

impl<'s, T> join::ElementFetcher for SparseSetFetch<'s, T> {
    type Element = &'s T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        self.entity_to_index
            .get(entity)
            .map(|index| &self.data[*index])
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains(entity)
    }
}

pub struct SparseSetFetchMut<'s, T> {
    entity_to_index: &'s EntityToIndex,
    data: &'s mut Vec<T>,
}

impl<'s, T> join::ElementFetcher for SparseSetFetchMut<'s, T> {
    type Element = &'s mut T;

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        self.entity_to_index.get(entity).map(|index| {
            let s: &'s mut Self = unsafe { &mut *(self as *mut Self) };
            &mut s.data[*index]
        })
    }

    fn contains(&self, entity: Entity) -> bool {
        self.entity_to_index.contains(entity)
    }
}

#[derive(Default)]
struct EntityToIndex {
    m: HashMap<Entity, usize>, // todo: optimize performance
}

impl EntityToIndex {
    pub(crate) fn contains(&self, entity: Entity) -> bool {
        self.m.contains_key(&entity)
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&usize> {
        self.m.get(&entity)
    }

    pub(crate) fn insert(&mut self, entity: Entity, index: usize) -> Option<usize> {
        self.m.insert(entity, index)
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Option<usize> {
        self.m.remove(&entity)
    }
}
