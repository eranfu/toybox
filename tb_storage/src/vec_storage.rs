use std::mem::MaybeUninit;
use std::ptr;

use hibitset::{BitSet, DrainableBitSet};

use tb_core::Id;

use crate::{util, Storage};

pub struct VecStorage<D> {
    mask: BitSet,
    base_id: Option<Id>,
    data: Vec<MaybeUninit<D>>,
}

impl<D> Drop for VecStorage<D> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<D> Default for VecStorage<D> {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            base_id: Default::default(),
            data: Default::default(),
        }
    }
}

impl<D> Storage for VecStorage<D> {
    type Data = D;

    fn clear(&mut self) {
        unsafe {
            for id in self.mask.drain() {
                let index = util::get_index_with_base(self.base_id, id);
                let data = self.data.get_unchecked_mut(index).as_mut_ptr();
                ptr::drop_in_place(data);
            }
            self.data.set_len(0);
        }
        self.base_id = None;
    }

    fn insert(&mut self, id: Id, data: D) -> &mut D {
        assert!(!self.mask.contains(id));
        self.mask.add(id);
        unsafe {
            let index = util::setup_index_with_base(&mut self.base_id, &mut self.data, id);
            self.data.get_unchecked_mut(index).as_mut_ptr().write(data);
            &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
        }
    }

    fn remove(&mut self, id: Id) {
        assert!(self.mask.contains(id));
        self.mask.remove(id);
        let index = util::get_index_with_base(self.base_id, id);
        unsafe {
            ptr::drop_in_place(self.data.get_unchecked_mut(index).as_mut_ptr());
        }
    }

    fn get(&self, id: Id) -> &D {
        assert!(self.mask.contains(id));
        let index = util::get_index_with_base(self.base_id, id);
        unsafe { &*self.data.get_unchecked(index).as_ptr() }
    }

    fn get_mut(&mut self, id: Id) -> &mut D {
        assert!(self.mask.contains(id));
        let index = util::get_index_with_base(self.base_id, id);
        unsafe { &mut *self.data.get_unchecked_mut(index).as_mut_ptr() }
    }

    fn contains(&self, id: u32) -> bool {
        self.mask.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use testdrop::{Item, TestDrop};

    use tb_core::Id;

    use crate::{Storage, VecStorage};

    #[derive(Debug)]
    struct DropItemData<'a> {
        id: Id,
        td: &'a TestDrop,
        drop_item: Item<'a>,
    }

    impl<'a> DropItemData<'a> {
        fn new(id: Id, td: &'a TestDrop) -> Self {
            Self {
                id,
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
            let data_4 = DropItemData::new(4, &td);
            let data_3 = DropItemData::new(3, &td);
            let data_2 = DropItemData::new(2, &td);
            let data_8 = DropItemData::new(8, &td);
            let data_6 = DropItemData::new(6, &td);
            storage.insert(4, data_4.clone());
            storage.insert(3, data_3.clone());
            storage.insert(2, data_2.clone());
            storage.insert(8, data_8.clone());
            storage.insert(6, data_6.clone());
            assert!(storage.contains(3));
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[1].as_ptr(), &data_3);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.data[6].as_ptr(), &data_8);

            storage.remove(3);
            assert!(!storage.contains(3));
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.data[6].as_ptr(), &data_8);

            storage.remove(8);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);

            drop(storage);
            assert_eq!(10, td.num_tracked_items());
            assert_eq!(5, td.num_dropped_items());
        }
    }

    #[test]
    #[should_panic(expected = "assertion failed: !self.mask.contains(id)")]
    fn duplicate_insert() {
        let mut storage = VecStorage::default();
        storage.insert(3, 3);
        storage.insert(3, 5);
    }
}
