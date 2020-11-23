use hibitset::BitSet;

use tb_core::Id;

use crate::{Storage, StorageItems};

pub struct TagStorageItems<T: Default> {
    data: T,
}

pub type TagStorage<D> = Storage<TagStorageItems<D>>;

impl<T: Default> Default for TagStorageItems<T> {
    fn default() -> Self {
        assert_eq!(std::mem::size_of::<T>(), 0);
        Self {
            data: Default::default(),
        }
    }
}

impl<T: Default> StorageItems for TagStorageItems<T> {
    type Data = T;

    unsafe fn clear(&mut self, _mask: &BitSet) {}

    unsafe fn insert(&mut self, _id: Id, _data: Self::Data) -> &mut Self::Data {
        &mut self.data
    }

    unsafe fn remove(&mut self, _id: Id) {}

    unsafe fn get(&self, _id: Id) -> &Self::Data {
        &self.data
    }

    unsafe fn get_mut(&mut self, _id: Id) -> &mut Self::Data {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use crate::TagStorage;

    #[derive(Default)]
    struct Tag;

    #[test]
    fn it_works() {
        let mut storage = TagStorage::default();
        storage.insert(2.into(), Tag);
        assert!(storage.contains(2.into()));
        assert!(!storage.contains(3.into()));
        storage.insert(3.into(), Tag);
        assert!(storage.contains(3.into()));
        storage.remove(2.into());
        assert!(!storage.contains(2.into()));
    }

    #[test]
    #[should_panic(expected = "assertion failed: !self.mask.add(*id)")]
    fn duplicate_insert() {
        let mut storage = TagStorage::default();
        storage.insert(2.into(), Tag);
        storage.insert(2.into(), Tag);
    }

    #[derive(Default)]
    struct NotZeroSizeTag {
        _id: usize,
    }

    #[test]
    #[should_panic(expected = "assertion failed: `(left == right)")]
    fn assert_zero_size() {
        TagStorage::<NotZeroSizeTag>::default();
    }
}
