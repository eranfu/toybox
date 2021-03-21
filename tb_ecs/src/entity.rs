use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bit_set::BitSet;

use crate::{Component, ComponentIndex, SystemData, World, Write, WriteComponents};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Entity {
    id: u64,
}

impl Entity {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Eq, PartialEq, Clone, Default, Hash)]
struct ComponentMask(BitSet<usize>);

#[derive(Default)]
pub struct Entities {
    next_id: u64,
    len: usize,
    entity_to_index: HashMap<Entity, (usize, usize)>,
    component_mask_to_archetype_index: HashMap<ComponentMask, usize>,
    archetypes_entities: Vec<Vec<Entity>>,
    archetypes_components: Vec<ComponentMask>,
    archetypes_add_to_next: Vec<HashMap<ComponentIndex, usize>>,
    archetypes_remove_to_next: Vec<HashMap<ComponentIndex, usize>>,
}

impl Entities {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub fn new_entity(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let entity = Entity { id };
        let archetype = self.get_or_insert_archetype(ComponentMask::default());
        self.entity_to_index
            .insert(entity, (archetype, self.push_entity(archetype, entity)));
        self.len += 1;
        entity
    }

    ///
    /// # return
    /// entity index in archetype
    fn push_entity(&mut self, archetype: usize, entity: Entity) -> usize {
        let entities = &mut self.archetypes_entities[archetype];
        let entity_index = entities.len();
        entities.push(entity);
        entity_index
    }

    fn get_or_insert_archetype(&mut self, mask: ComponentMask) -> usize {
        match self.component_mask_to_archetype_index.entry(mask) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let archetype = self.archetypes_components.len();
                vacant.insert(archetype);
                self.archetypes_components.push(vacant.key().clone());
                self.archetypes_entities.push(Default::default());
                self.archetypes_add_to_next.push(Default::default());
                self.archetypes_remove_to_next.push(Default::default());
                archetype
            }
        }
    }

    pub fn kill(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }

        self.archetypes.get_of_entity_mut(entity).remove(entity);
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn iter(&self) -> EntitiesIter {
        EntitiesIter {
            inner: self.entities.iter(),
        }
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
        let components = WriteComponents::<C>::fetch(self.world);
        components..insert(self.entity, c);

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
            self.world.fetch_mut::<Entities>().kill(self.entity);
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
    inner: SparseSetEntityIter<'e>,
}

impl<'e> Iterator for EntitiesIter<'e> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entities;
    use crate::{Entity, World};

    #[test]
    fn entity_life() {
        let mut entities = Entities::default();
        let entity0 = entities.new_entity();
        assert_eq!(entity0.id, 0);
        let entity1 = entities.new_entity();
        assert_eq!(entity1.id, 1);
        assert!(entities.is_alive(entity0));
        assert!(entities.is_alive(entity1));
        entities.kill(entity0);
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
        assert!(world.fetch::<Entities>().is_alive(entity));
        world.create_entity();
        let entities = world.fetch::<Entities>();
        assert!(!entities.entities.contains(Entity { id: 1 }));
    }
}
