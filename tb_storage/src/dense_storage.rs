use std::mem::MaybeUninit;

use hibitset::BitSet;

use tb_core::Id;

use crate::{util, Storage};

pub struct DenseStorage<D> {
    mask: BitSet,
    base_id: Option<Id>,
    indices: Vec<MaybeUninit<usize>>,
    data: Vec<D>,
    data_id: Vec<Id>,
}

impl<D> Default for DenseStorage<D> {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            base_id: Default::default(),
            indices: Default::default(),
            data: Default::default(),
            data_id: Default::default(),
        }
    }
}

impl<D> Storage for DenseStorage<D> {
    type Data = D;

    fn clear(&mut self) {
        self.mask.clear();
        self.base_id = None;
        self.data.clear();
        unsafe {
            self.data_id.set_len(0);
            self.indices.set_len(0);
        }
    }

    fn insert(&mut self, id: Id, data: D) -> &mut D {
        assert!(!self.mask.contains(*id));

        let index_in_data = self.data.len();
        self.data.push(data);
        self.data_id.push(id);
        self.mask.add(*id);

        unsafe {
            let index_in_indices =
                util::setup_index_with_base(&mut self.base_id, &mut self.indices, id);
            self.indices
                .get_unchecked_mut(index_in_indices)
                .as_mut_ptr()
                .write(index_in_data);
            self.data.get_unchecked_mut(index_in_data)
        }
    }

    fn remove(&mut self, id: Id) {
        assert!(self.mask.contains(*id));
        let index_in_indices = util::get_index_with_base(self.base_id, id);
        let index_in_data = unsafe { self.indices.get_unchecked(index_in_indices).assume_init() };
        let last_data_id = *self.data_id.last().unwrap();
        let last_data_index_in_indices = util::get_index_with_base(self.base_id, last_data_id);

        self.data.swap_remove(index_in_data);
        self.data_id.swap_remove(index_in_data);
        self.mask.remove(*id);

        unsafe {
            self.indices
                .get_unchecked_mut(last_data_index_in_indices)
                .as_mut_ptr()
                .write(index_in_data);
        }
    }

    fn get(&self, id: Id) -> &D {
        assert!(self.mask.contains(*id));
        unsafe {
            let index = self
                .indices
                .get_unchecked(util::get_index_with_base(self.base_id, id))
                .assume_init();
            self.data.get_unchecked(index)
        }
    }

    fn get_mut(&self, id: Id) -> &mut D {
        assert!(self.mask.contains(*id));
        unsafe {
            let index = self
                .indices
                .get_unchecked(util::get_index_with_base(self.base_id, id))
                .assume_init();
            &mut *(self.data.get_unchecked(index) as *const D as *mut D)
        }
    }

    fn contains(&self, id: Id) -> bool {
        self.mask.contains(*id)
    }

    fn mask(&self) -> &BitSet {
        &self.mask
    }
}

#[cfg(test)]
mod tests {
    use testdrop::{Item, TestDrop};

    use tb_core::Id;

    use crate::{DenseStorage, Storage};

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

    #[test]
    fn drop() {
        let td = TestDrop::new();
        let mut storage: DenseStorage<DropItemData> = Default::default();
        let data_4 = DropItemData::new(4.into(), &td);
        let data_3 = DropItemData::new(3.into(), &td);
        let data_2 = DropItemData::new(2.into(), &td);
        let data_8 = DropItemData::new(8.into(), &td);
        let data_6 = DropItemData::new(6.into(), &td);

        storage.insert(4.into(), data_4);
        storage.insert(3.into(), data_3);
        storage.insert(2.into(), data_2);
        storage.insert(8.into(), data_8);
        storage.insert(6.into(), data_6);

        storage.clear();
        assert_eq!(5, td.num_tracked_items());
        assert_eq!(5, td.num_dropped_items());
    }

    #[test]
    fn insert() {
        unsafe {
            let mut storage = DenseStorage::<i32>::default();
            assert_eq!(*storage.insert(3.into(), 3), 3);
            assert_eq!(storage.indices.len(), 1);
            assert_eq!(*storage.get(3.into()), 3);
            assert_eq!(*storage.get_mut(3.into()), 3);
            assert_eq!(*storage.insert(1.into(), 1), 1);
            assert_eq!(storage.indices.len(), 3);
            assert_eq!(storage.indices.get_unchecked_mut(0).assume_init(), 1);
            assert_eq!(storage.indices.get_unchecked_mut(2).assume_init(), 0);
            assert_eq!(*storage.get(1.into()), 1);
            assert_eq!(*storage.get(3.into()), 3);
            assert_eq!(*storage.insert(0.into(), 0), 0);
            assert_eq!(storage.indices.len(), 4);
            assert_eq!(*storage.get(1.into()), 1);
            assert_eq!(*storage.get(3.into()), 3);
            assert_eq!(*storage.get(0.into()), 0);
            assert!(storage.contains(3.into()));

            let mut storage = DenseStorage::<i32>::default();
            storage.insert(4.into(), 4);
            storage.insert(3.into(), 3);
            storage.insert(2.into(), 2);
            storage.insert(8.into(), 8);
            storage.insert(6.into(), 6);
            assert_eq!(storage.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(
                storage.data_id,
                vec![4.into(), 3.into(), 2.into(), 8.into(), 6.into()]
            );
            assert_eq!(storage.base_id, Some(2.into()));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[1].assume_init(), 1);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 4);
            assert_eq!(storage.indices[6].assume_init(), 3);
        }
    }

    #[test]
    fn remove() {
        unsafe {
            let mut storage = DenseStorage::<u32>::default();
            storage.insert(4.into(), 4);
            storage.insert(3.into(), 3);
            storage.insert(2.into(), 2);
            storage.insert(8.into(), 8);
            storage.insert(6.into(), 6);
            assert_eq!(storage.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(
                storage.data_id,
                vec![4.into(), 3.into(), 2.into(), 8.into(), 6.into()]
            );
            assert_eq!(storage.base_id, Some(2.into()));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[1].assume_init(), 1);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 4);
            assert_eq!(storage.indices[6].assume_init(), 3);
            assert!(storage.contains(3.into()));

            storage.remove(3.into());
            assert!(!storage.contains(3.into()));
            assert_eq!(storage.data, vec![4, 6, 2, 8]);
            assert_eq!(
                storage.data_id,
                vec![4.into(), 6.into(), 2.into(), 8.into()]
            );
            assert_eq!(storage.base_id, Some(2.into()));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 1);
            assert_eq!(storage.indices[6].assume_init(), 3);

            storage.remove(8.into());
            assert_eq!(storage.data, vec![4, 6, 2]);
            assert_eq!(storage.data_id, vec![4.into(), 6.into(), 2.into()]);
            assert_eq!(storage.base_id, Some(2.into()));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 1);
        }
    }
}
