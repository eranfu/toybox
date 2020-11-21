pub use dense_storage::DenseStorage;
use tb_core::Id;
pub use vec_storage::VecStorage;

pub trait Storage<D> {
    fn clear(&mut self);
    fn insert(&mut self, id: Id, data: D) -> &mut D;
    fn remove(&mut self, id: Id);
    fn get(&self, id: Id) -> &D;
    fn get_mut(&mut self, id: Id) -> &mut D;
}

mod dense_storage;
mod util;
mod vec_storage;
