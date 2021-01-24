use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::sparse_set::{SparseSet, SparseSetEntityIter};
use crate::{Component, World};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Entity {
    id: u64,
}

impl Entity {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(Default)]
pub struct Entities {
    entities: SparseSet<()>,
    next_id: AtomicU64,
}

pub struct EntityCreator<'r> {
    created: bool,
    entity: Entity,
    world: &'r mut World,
}

impl Deref for Entities {
    type Target = SparseSet<()>;

    fn deref(&self) -> &Self::Target {
        &self.entities
    }
}

impl EntityCreator<'_> {
    pub fn with<C: Component>(&mut self, c: C) -> &mut Self {
        let storage = self.world.insert_storage::<C>();
        storage.insert(self.entity, c);
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

impl Entities {
    pub fn new_entity(&mut self) -> Entity {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entity = Entity { id };
        self.entities.insert(entity, ());
        entity
    }

    pub fn kill(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }

        self.entities.remove(entity);
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
