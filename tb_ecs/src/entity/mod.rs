use generation::Generation;

mod generation;

pub(crate) type EntityId = usize;

pub struct Entity {
    id: EntityId,
    gen: Generation,
}

pub(crate) struct Entities {}