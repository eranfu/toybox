use std::mem::MaybeUninit;
use std::ptr;

use hibitset::{BitSet, BitSetLike};

use tb_core::Id;

use crate::{util, Storage, StorageItems};

pub struct VecStorageItems<D> {
    base_id: Option<Id>,
    data: Vec<MaybeUninit<D>>,
}

pub type VecStorage<D> = Storage<VecStorageItems<D>>;

impl<D> Default for VecStorageItems<D> {
    fn default() -> Self {
        Self {
            base_id: Default::default(),
            data: Default::default(),
        }
    }
}

impl<D> StorageItems for VecStorageItems<D> {
    type Data = D;

    unsafe fn clear(&mut self, mask: &BitSet) {
        for id in mask.iter() {
            let index = util::get_index_with_base(self.base_id, id.into());
            let data = self.data.get_unchecked_mut(index).as_mut_ptr();
            ptr::drop_in_place(data);
        }
        self.data.set_len(0);
        self.base_id = None;
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        let index = util::setup_index_with_base(&mut self.base_id, &mut self.data, id);
        self.data.get_unchecked_mut(index).as_mut_ptr().write(data);
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }

    unsafe fn remove(&mut self, id: Id) {
        let index = util::get_index_with_base(self.base_id, id);

        ptr::drop_in_place(self.data.get_unchecked_mut(index).as_mut_ptr());
    }

    unsafe fn get(&self, id: Id) -> &D {
        let index = util::get_index_with_base(self.base_id, id);
        &*self.data.get_unchecked(index).as_ptr()
    }

    unsafe fn get_mut(&mut self, id: Id) -> &mut D {
        let index = util::get_index_with_base(self.base_id, id);
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {
    use testdrop::{Item, TestDrop};

    use tb_core::Id;

    use crate::VecStorage;

    #[derive(Debug)]
    struct DropItemData<'a> {
        id: Id,
        td: &'a TestDrop,
        drop_item: Item<'a>,
    }

    impl<'a> DropItemData<'a> {
        fn new(id: impl Into<Id>, td: &'a TestDrop) -> Self {
            Self {
                id: id.into(),
                td,
                drop_item: td.new_item().1,
            }
        }
    }

    impl<'a> Drop for DropItemData<'a> {
        fn drop(&mut self) {
            println!("TestData in VecStorage dropped. id: {}", self.id);
        }
    }

    impl<'a> PartialEq for DropItemData<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl<'a> Eq for DropItemData<'a> {}

    impl<'a> Clone for DropItemData<'a> {
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
        unsafe {
            let td = TestDrop::new();
            let mut storage = VecStorage::<DropItemData>::default();
            let data_4 = DropItemData::new(4u32, &td);
            let data_3 = DropItemData::new(3u32, &td);
            let data_2 = DropItemData::new(2u32, &td);
            let data_8 = DropItemData::new(8u32, &td);
            let data_6 = DropItemData::new(6u32, &td);
            storage.insert(4u32.into(), data_4.clone());
            storage.insert(3u32.into(), data_3.clone());
            storage.insert(2u32.into(), data_2.clone());
            storage.insert(8u32.into(), data_8.clone());
            storage.insert(6u32.into(), data_6.clone());
            assert!(storage.contains(3u32.into()));
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(&*storage.items.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.items.data[1].as_ptr(), &data_3);
            assert_eq!(&*storage.items.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.items.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.items.data[6].as_ptr(), &data_8);

            storage.remove(3u32.into());
            assert!(!storage.contains(3u32.into()));
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(&*storage.items.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.items.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.items.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.items.data[6].as_ptr(), &data_8);

            storage.remove(8u32.into());
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(&*storage.items.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.items.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.items.data[4].as_ptr(), &data_6);

            drop(storage);
            assert_eq!(10, td.num_tracked_items());
            assert_eq!(5, td.num_dropped_items());
        }
    }

    #[test]
    #[should_panic(expected = "assertion failed: !self.mask.add(*id)")]
    fn duplicate_insert() {
        let mut storage = VecStorage::default();
        storage.insert(3u32.into(), 3);
        storage.insert(3u32.into(), 5);
    }
}
