use std::collections::hash_map::Entry;
use std::collections::HashMap;

use hibitset::BitSet;

use tb_core::Id;

use crate::{Storage, StorageItems};

pub struct HashMapStorageItems<D> {
    data: HashMap<Id, D>,
}

pub type HashMapStorage<D> = Storage<HashMapStorageItems<D>>;

impl<D> Default for HashMapStorageItems<D> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<D> StorageItems for HashMapStorageItems<D> {
    type Data = D;

    unsafe fn clear(&mut self, _mask: &BitSet) {
        self.data.clear();
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        match self.data.entry(id) {
            Entry::Occupied(_) => unreachable!(),
            Entry::Vacant(entry) => entry.insert(data),
        }
    }

    unsafe fn remove(&mut self, id: Id) {
        self.data.remove(&id).unwrap();
    }

    unsafe fn get(&self, id: Id) -> &D {
        self.data.get(&id).unwrap()
    }

    unsafe fn get_mut(&mut self, id: Id) -> &mut D {
        self.data.get_mut(&id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use testdrop::{Item, TestDrop};

    use tb_core::Id;

    use crate::HashMapStorage;

    #[derive(Debug)]
    struct HashMapData<'a> {
        id: Id,
        td: &'a TestDrop,
        drop_item: Item<'a>,
    }

    impl<'a> HashMapData<'a> {
        fn new(id: impl Into<Id>, td: &'a TestDrop) -> Self {
            Self {
                id: id.into(),
                td,
                drop_item: td.new_item().1,
            }
        }
    }

    impl<'a> Drop for HashMapData<'a> {
        fn drop(&mut self) {
            println!("TestData in HashMapStorage dropped. id: {}", self.id);
        }
    }

    impl<'a> PartialEq for HashMapData<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl<'a> Eq for HashMapData<'a> {}

    impl<'a> Clone for HashMapData<'a> {
        fn clone(&self) -> Self {
            Self {
                id: self.id,
                td: self.td,
                drop_item: self.td.new_item().1,
            }
        }
    }

    #[test]
    fn it_works() {
        let td = TestDrop::new();
        let mut storage = HashMapStorage::<HashMapData>::default();
        let data_4 = HashMapData::new(4u32, &td);
        let data_3 = HashMapData::new(3u32, &td);
        let data_2 = HashMapData::new(2u32, &td);
        let data_8 = HashMapData::new(8u32, &td);
        let data_6 = HashMapData::new(6u32, &td);
        storage.insert(4u32.into(), data_4.clone());
        storage.insert(3u32.into(), data_3.clone());
        storage.insert(2u32.into(), data_2.clone());
        storage.insert(8u32.into(), data_8.clone());
        storage.insert(6u32.into(), data_6.clone());
        assert!(storage.contains(3u32.into()));
        assert_eq!(&*storage.get(2.into()), &data_2);
        assert_eq!(&*storage.get(3.into()), &data_3);
        assert_eq!(&*storage.get(4.into()), &data_4);
        assert_eq!(&*storage.get(6.into()), &data_6);
        assert_eq!(&*storage.get(8.into()), &data_8);

        storage.remove(3u32.into());
        assert!(!storage.contains(3u32.into()));
        assert_eq!(&*storage.get(2.into()), &data_2);
        assert_eq!(&*storage.get(4.into()), &data_4);
        assert_eq!(&*storage.get(6.into()), &data_6);
        assert_eq!(&*storage.get(8.into()), &data_8);

        storage.remove(8u32.into());
        assert_eq!(&*storage.get(2.into()), &data_2);
        assert_eq!(&*storage.get(4.into()), &data_4);
        assert_eq!(&*storage.get(6.into()), &data_6);

        drop(storage);
        assert_eq!(10, td.num_tracked_items());
        assert_eq!(5, td.num_dropped_items());
    }

    #[test]
    #[should_panic(expected = "assertion failed: !self.mask.add(*id)")]
    fn duplicate_insert() {
        let mut storage = HashMapStorage::default();
        storage.insert(3u32.into(), 3);
        storage.insert(3u32.into(), 5);
    }
}
