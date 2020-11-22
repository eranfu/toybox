use hibitset::BitSet;

pub use dense_storage::DenseStorage;
pub use tag_storage::TagStorage;
use tb_core::Id;
pub use vec_storage::VecStorage;

pub trait Storage {
    type Data;
    fn clear(&mut self);
    fn insert(&mut self, id: Id, data: Self::Data) -> &mut Self::Data;
    fn remove(&mut self, id: Id);
    fn get(&self, id: Id) -> &Self::Data;
    fn get_mut(&self, id: Id) -> &mut Self::Data;
    fn contains(&self, id: Id) -> bool;
    fn mask(&self) -> &BitSet;
}

mod dense_storage;
mod tag_storage;
mod util;
mod vec_storage;
