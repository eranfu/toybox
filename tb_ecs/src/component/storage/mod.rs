pub use dense_storage::DenseStorage;

pub(crate) trait Storage {}

pub struct RBWStorage<'r, S: Storage> {}


mod dense_storage;