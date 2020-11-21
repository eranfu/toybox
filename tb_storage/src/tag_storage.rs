use hibitset::BitSet;

use tb_core::Id;

use crate::Storage;

pub struct TagStorage<T: Default> {
    mask: BitSet,
    data: T,
}

impl<T: Default> Default for TagStorage<T> {
    fn default() -> Self {
        assert_eq!(std::mem::size_of::<T>(), 0);
        Self {
            mask: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T: Default> Storage for TagStorage<T> {
    type Data = T;

    fn clear(&mut self) {
        self.mask.clear();
    }

    fn insert(&mut self, id: Id, _data: Self::Data) -> &mut Self::Data {
        assert!(!self.mask.add(id));
        &mut self.data
    }

    fn remove(&mut self, id: Id) {
        assert!(self.mask.remove(id));
    }

    fn get(&self, id: Id) -> &Self::Data {
        assert!(self.mask.contains(id));
        &self.data
    }

    fn get_mut(&mut self, id: Id) -> &mut Self::Data {
        assert!(self.mask.contains(id));
        &mut self.data
    }

    fn contains(&self, id: Id) -> bool {
        self.mask.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Storage, TagStorage};

    #[derive(Default)]
    struct Tag;

    #[test]
    fn it_works() {
        let mut storage = TagStorage::default();
        storage.insert(2, Tag);
        assert!(storage.contains(2));
        assert!(!storage.contains(3));
        storage.insert(3, Tag);
        assert!(storage.contains(3));
        storage.remove(2);
        assert!(!storage.contains(2));
    }

    #[test]
    #[should_panic(expected = "assertion failed: !self.mask.add(id)")]
    fn duplicate_insert() {
        let mut storage = TagStorage::default();
        storage.insert(2, Tag);
        storage.insert(2, Tag);
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
