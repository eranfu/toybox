use std::mem::MaybeUninit;
use std::ptr;

use hibitset::BitSetLike;

use tb_core::Id;

use crate::{util, Storage};

pub struct VecStorage<D> {
    data: Vec<MaybeUninit<D>>,
    base_id: Option<Id>,
}

impl<D> VecStorage<D> {
    fn get_index(&self, id: Id) -> usize {
        id.checked_sub(self.base_id.unwrap()).unwrap() as usize
    }
}

impl<D> Default for VecStorage<D> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            base_id: Default::default(),
        }
    }
}

impl<D> Storage<D> for VecStorage<D> {
    unsafe fn clear<B: BitSetLike>(&mut self, has: B) {
        has.iter().for_each(|id| self.remove(id));
        self.data.set_len(0);
        self.base_id = None;
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        util::setup_base_id(&mut self.base_id, &mut self.data, id);
        let index = self.get_index(id);
        util::ensure_index(&mut self.data, index);
        self.data.get_unchecked_mut(index).as_mut_ptr().write(data);
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }

    unsafe fn remove(&mut self, id: Id) {
        let index = self.get_index(id);
        ptr::drop_in_place(self.data.get_unchecked_mut(index).as_mut_ptr());
    }

    unsafe fn get(&self, id: Id) -> &D {
        let index = self.get_index(id);
        &*self.data.get_unchecked(index).as_ptr()
    }

    unsafe fn get_mut(&mut self, id: Id) -> &mut D {
        let index = self.get_index(id);
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {
    use hibitset::BitSet;

    use tb_core::Id;

    use crate::{Storage, VecStorage};

    #[derive(Eq, PartialEq, Debug, Clone)]
    struct TestData {
        id: Id,
    }

    impl TestData {
        fn new(id: Id) -> Self {
            Self { id }
        }
    }

    impl Drop for TestData {
        fn drop(&mut self) {
            println!("TestData dropped. id: {}", self.id);
        }
    }

    #[test]
    fn it_works() {
        unsafe {
            let mut storage = VecStorage::<TestData>::default();
            let mut has = BitSet::new();
            let data_4 = TestData::new(4);
            let data_3 = TestData::new(3);
            let data_2 = TestData::new(2);
            let data_8 = TestData::new(8);
            let data_6 = TestData::new(6);
            has.add(4);
            has.add(3);
            has.add(2);
            has.add(8);
            has.add(6);
            storage.insert(4, data_4.clone());
            storage.insert(3, data_3.clone());
            storage.insert(2, data_2.clone());
            storage.insert(8, data_8.clone());
            storage.insert(6, data_6.clone());
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[1].as_ptr(), &data_3);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.data[6].as_ptr(), &data_8);

            storage.remove(3);
            has.remove(3);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);
            assert_eq!(&*storage.data[6].as_ptr(), &data_8);

            storage.remove(8);
            has.remove(8);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(&*storage.data[0].as_ptr(), &data_2);
            assert_eq!(&*storage.data[2].as_ptr(), &data_4);
            assert_eq!(&*storage.data[4].as_ptr(), &data_6);

            storage.clear(has);
        }
    }
}
