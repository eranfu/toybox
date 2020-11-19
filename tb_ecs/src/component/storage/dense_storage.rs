use crate::component::{Component, Components};
use crate::entity::EntityId;

pub struct DenseStorage<C: Component> {
    components: Vec<C>,
    entity_ids: Vec<EntityId>,
    indices_of_entity: Vec<usize>,
}

impl<C: Component> Components<C> for DenseStorage<C> {}
