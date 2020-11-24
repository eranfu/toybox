use hibitset::BitSet;

pub use dense_storage::DenseStorage;
pub use dense_storage::DenseStorageItems;
pub use tag_storage::TagStorage;
pub use tag_storage::TagStorageItems;
use tb_core::Id;
pub use vec_storage::VecStorage;
pub use vec_storage::VecStorageItems;

pub struct Storage<I: StorageItems> {
    mask: BitSet,
    items: I,
}

pub trait StorageItems: Default {
    type Data;
    /// # Safety
    ///
    /// Given mask must means which data there is.
    unsafe fn clear(&mut self, mask: &BitSet);

    /// # Safety
    ///
    /// There should be no data associated with given id.
    unsafe fn insert(&mut self, id: Id, data: Self::Data) -> &mut Self::Data;

    /// # Safety
    ///
    /// There should be data associated with given id.
    unsafe fn remove(&mut self, id: Id);

    /// # Safety
    ///
    /// There should be data associated with given id.
    unsafe fn get(&self, id: Id) -> &Self::Data;

    /// # Safety
    ///
    /// There should be data associated with given id.
    unsafe fn get_mut(&mut self, id: Id) -> &mut Self::Data;
}

impl<I: StorageItems> Storage<I> {
    pub fn clear(&mut self) {
        unsafe { self.items.clear(&self.mask) }
    }
    pub fn open(&self) -> (&BitSet, &I) {
        (&self.mask, &self.items)
    }
    pub fn open_mut(&mut self) -> (&BitSet, &mut I) {
        (&self.mask, &mut self.items)
    }
    pub fn contains(&self, id: Id) -> bool {
        self.mask.contains(*id)
    }
    pub fn insert(&mut self, id: Id, data: I::Data) -> &mut I::Data {
        assert!(!self.mask.add(*id));
        unsafe { self.items.insert(id, data) }
    }
    pub fn remove(&mut self, id: Id) {
        assert!(self.mask.remove(*id));
        unsafe { self.items.remove(id) }
    }
    pub fn get(&self, id: Id) -> &I::Data {
        assert!(self.contains(id));
        unsafe { self.items.get(id) }
    }
    pub fn get_mut(&mut self, id: Id) -> &mut I::Data {
        assert!(self.contains(id));
        unsafe { self.items.get_mut(id) }
    }
}

impl<I: StorageItems> Drop for Storage<I> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<I: StorageItems> Default for Storage<I> {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            items: Default::default(),
        }
    }
}

mod dense_storage;
mod tag_storage;
mod util;
mod vec_storage;
