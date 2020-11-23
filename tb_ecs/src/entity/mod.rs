use generation::Generation;
use tb_storage::VecStorage;

mod generation;

pub(crate) type EntityId = usize;

pub struct Entity {
    id: EntityId,
    gen: Generation,
}

pub(crate) struct Entities {
    generations: VecStorage<Generation>,
}

impl Entities {}
