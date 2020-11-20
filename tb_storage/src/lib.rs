pub use dense_storage::DenseStorage;
use tb_core::Id;

pub trait Storage<D> {
    fn clear(&mut self);

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
