use hibitset::{BitSet, BitSetLike};

use generation::Generation;
use tb_core::Id;

use crate::{Component, World};

mod generation;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Entity {
    id: Id,
    gen: Generation,
}

#[derive(Default)]
pub struct Entities {
    alive: BitSet,
    killed: BitSet,
    generations: Vec<Generation>,
}

pub struct EntityCreator<'r> {
    created: bool,
    entity: Entity,
    world: &'r mut World,
}

impl Entity {
    pub fn id(&self) -> Id {
        self.id
    }
}

impl EntityCreator<'_> {
    pub fn with<C: Component>(&mut self, c: C) -> &mut Self {
        let storage = self.world.fetch_or_insert_storage::<C>();
        storage.insert(self.entity.id, c);
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
        let entity = self.fetch_or_insert_default::<Entities>().new_entity();
        EntityCreator {
            created: false,
            entity,
            world: self,
        }
    }
}

impl Entities {
    pub fn new_entity(&mut self) -> Entity {
        match (&self.killed).iter().next() {
            None => {
                let id: Id = self.generations.len().into();
                self.generations.push(Generation::new_alive());
                self.alive.add(*id);
                Entity {
                    id,
                    gen: unsafe { *self.generations.get_unchecked(id.as_usize()) },
                }
            }
            Some(id) => {
                let gen = unsafe { self.generations.get_unchecked_mut(id as usize) };
                gen.relive();
                self.killed.remove(id);
                self.alive.add(id);
                Entity {
                    id: id.into(),
                    gen: *gen,
                }
            }
        }
    }

    pub fn kill(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        unsafe {
            self.generations
                .get_unchecked_mut(*entity.id as usize)
                .die();
        }
        self.killed.add(*entity.id);
        self.alive.remove(*entity.id);
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        entity.gen.is_alive()
            && self.alive.contains(*entity.id)
            && unsafe { *self.generations.get_unchecked(*entity.id as usize) == entity.gen }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entities;
    use crate::World;

    #[test]
    fn entity_life() {
        let mut entities = Entities::default();
        let entity0 = entities.new_entity();
        assert_eq!(*entity0.id, 0);
        let entity1 = entities.new_entity();
        assert_eq!(*entity1.id, 1);
        assert!(entities.is_alive(entity0));
        assert!(entities.is_alive(entity1));
        entities.kill(entity0);
        assert!(!entities.is_alive(entity0));
        let entity0 = entities.new_entity();
        assert_eq!(*entity0.id, 0);
        assert!(entities.is_alive(entity0));
    }

    #[test]
    fn create_entity_failed() {
        let mut world = World::default();
        let entity = world.create_entity().create();
        assert_eq!(*entity.id, 0);
        assert!(world.fetch::<Entities>().is_alive(entity));
        world.create_entity();
        let entities = world.fetch::<Entities>();
        assert!(entities.killed.contains(1));
        assert!(!entities.alive.contains(1));
        assert!(!entities.generations.get(1).unwrap().is_alive())
    }
}
