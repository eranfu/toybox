use generation::Generation;

mod generation;

pub struct Entity {
    id: usize,
    gen: Generation,
}