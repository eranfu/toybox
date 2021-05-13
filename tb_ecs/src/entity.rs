use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::{Copied, Flatten, Fuse};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::Iter;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use bit_set::BitSet;
use serde::{Deserialize, Serialize};

use crate::registry::{ComponentIndex, ComponentRegistry};
use crate::{Component, Join, SystemData, World, WriteComponents};

#[derive(Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Entity {
    id: u64,
}

impl Entity {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Eq, PartialEq, Clone, Default, Hash)]
pub(crate) struct ComponentMask(BitSet<usize>);

impl Deref for ComponentMask {
    type Target = BitSet<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ComponentMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct Entities {
    inner: RwLock<EntitiesInner>,
}

impl Entities {
    pub fn len(&self) -> usize {
        self.read().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn iter(&self) -> EntitiesIter<'_> {
        EntitiesIter::new(self.read())
    }
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.read().is_alive(entity)
    }
    pub fn new_entity(&self) -> Entity {
        self.write().new_entity()
    }
    pub(crate) unsafe fn kill(&self, world: &World, entity: Entity) {
        self.write().kill(world, entity)
    }
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, EntitiesInner> {
        self.inner.read().unwrap()
    }
    fn write(&self) -> RwLockWriteGuard<'_, EntitiesInner> {
        self.inner.write().unwrap()
    }
    pub(crate) fn on_component_inserted<C: Component>(&self, entity: Entity) {
        let component_index = ComponentIndex::get::<C>();
        self.inner
            .write()
            .unwrap()
            .on_component_inserted(entity, component_index);
    }
}

#[derive(Default)]
pub struct EntitiesInner {
    next_id: u64,
    len: usize,
    matched_entities_map: RwLock<HashMap<TypeId, RwLock<MatchedEntities>>>,
    entity_to_index: HashMap<Entity, EntityIndex>,
    component_mask_to_archetype_index: HashMap<ComponentMask, ArchetypeIndex>,
    archetypes_entities: Vec<Vec<Entity>>,
    archetypes_component_mask: Vec<ComponentMask>,
    archetypes_add_to_next: Vec<HashMap<ComponentIndex, ArchetypeIndex>>,
    archetypes_remove_to_next: Vec<HashMap<ComponentIndex, ArchetypeIndex>>,
}

impl EntitiesInner {
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_to_index.contains_key(&entity)
    }

    pub fn iter(&self) -> Flatten<Iter<'_, Vec<Entity>>> {
        self.archetypes_entities.iter().flatten()
    }
    pub unsafe fn kill(&mut self, world: &World, entity: Entity) {
        let entity_index = match self.entity_to_index.remove(&entity) {
            Some(entity_index) => entity_index,
            None => {
                return;
            }
        };

        let entities = &mut self.archetypes_entities[entity_index.archetype];
        let last = *entities.last().unwrap();
        entities.swap_remove(entity_index.index_in_archetype);
        if last != entity {
            self.entity_to_index.insert(last, entity_index);
        }

        for component_index in self.archetypes_component_mask[entity_index.archetype].iter() {
            ComponentRegistry::operation(component_index.into())
                .0
                .remove_from_world(world, entity)
        }
    }

    pub(crate) fn visit_archetype(
        &self,
        visitor: &mut ArchetypeVisitor,
        mut on_visit: impl FnMut(ArchetypeIndex, &ComponentMask),
    ) {
        while *visitor.cur_index < self.archetypes_component_mask.len() {
            on_visit(
                visitor.cur_index,
                &self.archetypes_component_mask[visitor.cur_index],
            );
            *visitor.cur_index += 1;
        }
    }

    fn on_component_inserted(&mut self, entity: Entity, component_index: ComponentIndex) {
        let entity_index = match self.entity_to_index.get(&entity).copied() {
            Some(index) => index,
            None => {
                return;
            }
        };

        let next_archetype = self.archetypes_add_to_next[entity_index.archetype]
            .get(&component_index)
            .copied();
        let next_archetype = next_archetype.unwrap_or_else(|| {
            let mut next_mask = self.archetypes_component_mask[entity_index.archetype].clone();
            next_mask.insert(*component_index);
            let next_archetype = self.get_or_insert_archetype(next_mask);
            self.archetypes_add_to_next[entity_index.archetype]
                .insert(component_index, next_archetype);
            next_archetype
        });

        self.transfer(entity, entity_index, next_archetype);
    }

    fn transfer(&mut self, entity: Entity, from: EntityIndex, to: ArchetypeIndex) {
        let from_entities = &mut self.archetypes_entities[from.archetype];
        let from_last = *from_entities.last().unwrap();
        from_entities.swap_remove(from.index_in_archetype);
        if from_last != entity {
            self.entity_to_index.insert(from_last, from);
        }
        let to_entity_index = self.push_entity(to, entity);
        self.entity_to_index.insert(entity, to_entity_index);
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub fn new_entity(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let entity = Entity { id };
        let archetype = self.get_or_insert_archetype(ComponentMask::default());
        let new_entity_index = self.push_entity(archetype, entity);
        self.entity_to_index.insert(entity, new_entity_index);
        self.len += 1;
        entity
    }

    fn push_entity(&mut self, archetype: ArchetypeIndex, entity: Entity) -> EntityIndex {
        let entities = &mut self.archetypes_entities[archetype];
        let index_in_archetype = entities.len();
        entities.push(entity);
        EntityIndex::new(archetype, index_in_archetype)
    }

    fn get_or_insert_archetype(&mut self, mask: ComponentMask) -> ArchetypeIndex {
        match self.component_mask_to_archetype_index.entry(mask) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let archetype = ArchetypeIndex(self.archetypes_component_mask.len());
                self.archetypes_component_mask.push(vacant.key().clone());
                vacant.insert(archetype);
                self.archetypes_entities.push(Default::default());
                self.archetypes_add_to_next.push(Default::default());
                self.archetypes_remove_to_next.push(Default::default());
                archetype
            }
        }
    }
}

pub(crate) struct ArchetypeVisitor {
    pub(crate) cur_index: ArchetypeIndex,
}

#[derive(Copy, Clone)]
struct EntityIndex {
    archetype: ArchetypeIndex,
    index_in_archetype: usize,
}

impl EntityIndex {
    fn new(archetype: ArchetypeIndex, index_in_archetype: usize) -> EntityIndex {
        Self {
            archetype,
            index_in_archetype,
        }
    }
}

impl Index<EntityIndex> for Vec<Vec<Entity>> {
    type Output = Entity;

    fn index(&self, index: EntityIndex) -> &Self::Output {
        &self[index.archetype][index.index_in_archetype]
    }
}

impl IndexMut<EntityIndex> for Vec<Vec<Entity>> {
    fn index_mut(&mut self, index: EntityIndex) -> &mut Self::Output {
        &mut self[index.archetype][index.index_in_archetype]
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ArchetypeIndex(pub(crate) usize);

impl Deref for ArchetypeIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ArchetypeIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<ArchetypeIndex> for Vec<Vec<Entity>> {
    type Output = Vec<Entity>;

    fn index(&self, index: ArchetypeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<ArchetypeIndex> for Vec<Vec<Entity>> {
    fn index_mut(&mut self, index: ArchetypeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl Index<ArchetypeIndex> for Vec<ComponentMask> {
    type Output = ComponentMask;

    fn index(&self, index: ArchetypeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<ArchetypeIndex> for Vec<ComponentMask> {
    fn index_mut(&mut self, index: ArchetypeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl Index<ArchetypeIndex> for Vec<HashMap<ComponentIndex, ArchetypeIndex>> {
    type Output = HashMap<ComponentIndex, ArchetypeIndex>;

    fn index(&self, index: ArchetypeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<ArchetypeIndex> for Vec<HashMap<ComponentIndex, ArchetypeIndex>> {
    fn index_mut(&mut self, index: ArchetypeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

pub struct EntityCreator<'r> {
    created: bool,
    entity: Entity,
    world: &'r mut World,
}

impl EntityCreator<'_> {
    pub fn with<C: Component>(&mut self, c: C) -> &mut Self {
        self.world.insert_components::<C>();
        let mut components = unsafe { WriteComponents::<C>::fetch(self.world) };
        components.insert(self.entity, c);
        self
    }
    pub fn create(&mut self) -> Entity {
        self.created = true;
        self.entity
    }
}

impl Drop for EntityCreator<'_> {
    fn drop(&mut self) {
        if !self.created {
            unsafe {
                self.world.fetch::<Entities>().kill(self.world, self.entity);
            }
        }
    }
}

impl World {
    pub fn create_entity(&mut self) -> EntityCreator {
        let entity = self.insert(Entities::default).new_entity();
        EntityCreator {
            created: false,
            entity,
            world: self,
        }
    }
}

pub struct EntitiesIter<'e> {
    _guard: RwLockReadGuard<'e, EntitiesInner>,
    inner: Flatten<Iter<'e, Vec<Entity>>>,
}

impl<'e> EntitiesIter<'e> {
    fn new(guard: RwLockReadGuard<EntitiesInner>) -> EntitiesIter {
        let inner = unsafe { std::mem::transmute(guard.iter()) };
        EntitiesIter {
            _guard: guard,
            inner,
        }
    }
}

impl<'e> Iterator for EntitiesIter<'e> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

#[derive(Default)]
pub struct ArchetypeMatcher {
    all: ComponentMask,
    none: ComponentMask,
    // todo: any: ComponentMask,
}

impl ArchetypeMatcher {
    pub(crate) fn add_all(&mut self, component_index: ComponentIndex) {
        self.all.insert(*component_index);
    }
    pub(crate) fn add_none(&mut self, component_index: ComponentIndex) {
        self.none.insert(*component_index);
    }
    pub(crate) fn is_matched(&self, archetype_component_mask: &ComponentMask) -> bool {
        self.all.is_subset(archetype_component_mask)
            && self.none.is_disjoint(archetype_component_mask)
    }
}

struct MatchedEntities {
    matcher: ArchetypeMatcher,
    archetype_visitor: ArchetypeVisitor,
    matched_archetypes: Vec<ArchetypeIndex>,
}

impl MatchedEntities {
    fn get<'j, 'e, T: Join<'j>>(
        entities: &'e RwLockReadGuard<'e, EntitiesInner>,
    ) -> (
        RwLockReadGuard<'e, HashMap<TypeId, RwLock<MatchedEntities>>>,
        RwLockReadGuard<'e, MatchedEntities>,
    ) {
        let matched_entities_map = &entities.matched_entities_map;
        let t_id = TypeId::of::<T::Element>();
        let map_read = matched_entities_map.read().unwrap();
        let (map, matched): (_, &'static RwLock<MatchedEntities>) =
            if let Some(matched) = map_read.get(&t_id) {
                let matched = unsafe { std::mem::transmute(matched) };
                (map_read, matched)
            } else {
                drop(map_read);
                let mut map_write = matched_entities_map.write().unwrap();
                match map_write.entry(t_id) {
                    Entry::Occupied(_occupied) => {}
                    Entry::Vacant(vacant) => {
                        vacant.insert(RwLock::new(MatchedEntities {
                            matcher: T::create_matcher(),
                            archetype_visitor: ArchetypeVisitor {
                                cur_index: ArchetypeIndex(0),
                            },
                            matched_archetypes: vec![],
                        }));
                    }
                }
                drop(map_write);
                let map_read = matched_entities_map.read().unwrap();
                let matched = unsafe { std::mem::transmute(map_read.get(&t_id).unwrap()) };
                (map_read, matched)
            };
        let matched_read = matched.read().unwrap();
        let matched = if *matched_read.archetype_visitor.cur_index
            >= entities.archetypes_component_mask.len()
        {
            matched_read
        } else {
            drop(matched_read);
            let mut matched_write = matched.write().unwrap();
            matched_write.do_match(entities);
            drop(matched_write);
            let matched_read = matched.read().unwrap();
            matched_read
        };

        (map, matched)
    }

    fn do_match(&mut self, entities: &RwLockReadGuard<EntitiesInner>) {
        let matcher = &self.matcher;
        let matched_archetypes = &mut self.matched_archetypes;
        entities.visit_archetype(
            &mut self.archetype_visitor,
            |archetype, archetype_component_mask| {
                if matcher.is_matched(archetype_component_mask) {
                    matched_archetypes.push(archetype);
                }
            },
        );
    }
}

pub(crate) struct MatchedEntitiesIter<'e> {
    archetype_entities: Option<Copied<Iter<'e, Entity>>>,
    entities: RwLockReadGuard<'e, EntitiesInner>,
    archetypes: Fuse<Iter<'e, ArchetypeIndex>>,
    _matched_entities_map: RwLockReadGuard<'e, HashMap<TypeId, RwLock<MatchedEntities>>>,
    _matched_entities: RwLockReadGuard<'e, MatchedEntities>,
}

impl<'e> MatchedEntitiesIter<'e> {
    pub(crate) fn get<'j, T: Join<'j>>(entities: RwLockReadGuard<'e, EntitiesInner>) -> Self {
        let (matched_entities_map, matched_entities): (
            RwLockReadGuard<'e, HashMap<TypeId, RwLock<MatchedEntities>>>,
            RwLockReadGuard<'e, MatchedEntities>,
        ) = unsafe { std::mem::transmute(MatchedEntities::get::<T>(&entities)) };

        let archetypes =
            unsafe { std::mem::transmute(matched_entities.matched_archetypes.iter().fuse()) };

        Self {
            archetype_entities: None,
            entities,
            archetypes,
            _matched_entities_map: matched_entities_map,
            _matched_entities: matched_entities,
        }
    }
}

impl<'e> Iterator for MatchedEntitiesIter<'e> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.archetype_entities.as_mut() {
                None => {
                    self.archetype_entities = {
                        match self.archetypes.next() {
                            None => {
                                return None;
                            }
                            Some(&archetype) => {
                                let s: &MatchedEntitiesIter<'e> =
                                    unsafe { &*(self as &mut _ as *mut _) };
                                Some(s.entities.archetypes_entities[archetype].iter().copied())
                            }
                        }
                    }
                }
                Some(archetype_entities) => match archetype_entities.next() {
                    None => {
                        self.archetype_entities = None;
                    }
                    entity @ Some(_) => {
                        return entity;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entities;
    use crate::{Entity, World};

    #[test]
    fn entity_life() {
        let mut world = World::default();
        let entities = world.insert(Entities::default);
        let entity0 = entities.new_entity();
        assert_eq!(entity0.id, 0);
        let entity1 = entities.new_entity();
        assert_eq!(entity1.id, 1);
        assert!(entities.is_alive(entity0));
        assert!(entities.is_alive(entity1));
        let entities = unsafe { world.fetch::<Entities>() };
        unsafe {
            entities.kill(&world, entity0);
        }
        assert!(!entities.is_alive(entity0));
        let entity0 = entities.new_entity();
        assert_eq!(entity0.id, 2);
        assert!(entities.is_alive(entity0));
    }

    #[test]
    fn create_entity_failed() {
        let mut world = World::default();
        let entity = world.create_entity().create();
        assert_eq!(entity.id, 0);
        unsafe {
            assert!(world.fetch::<Entities>().is_alive(entity));
        }
        world.create_entity();
        let entities = unsafe { world.fetch::<Entities>() };
        assert!(!entities
            .read()
            .entity_to_index
            .contains_key(&Entity { id: 1 }));
    }
}
