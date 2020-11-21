use hibitset::BitSetLike;

pub use dense_storage::DenseStorage;
use tb_core::Id;
pub use vec_storage::VecStorage;

pub trait Storage<D> {
    /// Remove all data
    ///
    /// # Safety
    ///
    /// The given `has` should mean which data there is.
    unsafe fn clear<B: BitSetLike>(&mut self, has: B);

    /// Insert new data for a given `Id`.
    ///
    /// # Safety
    ///
    /// This function should not be called if there is data associated with given `Id` already.
    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D;

    /// Remove data associated with given `Id`
    ///
    /// # Safety
    ///
    /// This function should not be called if there is no data associated with given `Id`.
    unsafe fn remove(&mut self, id: Id);

    /// Get data associated with given `Id`
    ///
    /// # Safety
    ///
    /// There must be data associated with given `Id` already.
    unsafe fn get(&self, id: Id) -> &D;

    /// Get mutable data associated with given `Id`
    ///
    /// # Safety
    ///
    /// There must be data associated with given `Id` already.
    unsafe fn get_mut(&mut self, id: Id) -> &mut D;
}

mod dense_storage;
mod util;
mod vec_storage;
