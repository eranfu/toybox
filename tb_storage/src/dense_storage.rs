use std::mem::MaybeUninit;

use tb_core::Id;

use crate::{util, Storage};

#[derive(Default)]
pub struct DenseStorage<D> {
    data: Vec<D>,
    data_id: Vec<Id>,
    indices: Vec<MaybeUninit<usize>>,
    base_id: Option<Id>,
}

impl<D> DenseStorage<D> {
    fn get_index_in_indices(&self, id: u32) -> usize {
        id.checked_sub(self.base_id.unwrap()).unwrap() as usize
    }
}

impl<D> Storage<D> for DenseStorage<D> {
    fn clear(&mut self) {
        self.data.clear();
        self.data_id.clear();
        self.indices.clear();
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        util::setup_base_id(&mut self.base_id, &mut self.indices, id);
        let index_in_indices = self.get_index_in_indices(id);
        util::ensure_index(&mut self.indices, index_in_indices);

        let index_in_data = self.data.len();
        self.indices
            .get_unchecked_mut(index_in_indices)
            .as_mut_ptr()
            .write(index_in_data);
        self.data.push(data);
        self.data_id.push(id);

        self.data.get_unchecked_mut(index_in_data)
    }

    unsafe fn remove(&mut self, id: u32) {
        let index_in_indices = self.get_index_in_indices(id);
        let index_in_data = self.indices.get_unchecked(index_in_indices).assume_init();
        let last_data_id = *self.data_id.last().unwrap();
        let last_data_index_in_indices = self.get_index_in_indices(last_data_id);
        self.data.swap_remove(index_in_data);
        self.data_id.swap_remove(index_in_data);
        self.indices
            .get_unchecked_mut(last_data_index_in_indices)
            .as_mut_ptr()
            .write(index_in_data);
    }

    unsafe fn get(&self, id: u32) -> &D {
        let index = self
            .indices
            .get_unchecked(self.get_index_in_indices(id))
            .assume_init();
        self.data.get_unchecked(index)
    }

    unsafe fn get_mut(&mut self, id: u32) -> &mut D {
        let index = self
            .indices
            .get_unchecked(self.get_index_in_indices(id))
            .assume_init();
        self.data.get_unchecked_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DenseStorage, Storage};

    #[test]
    fn insert() {
        unsafe {
            let mut storage = DenseStorage::<i32>::default();
            assert_eq!(*storage.insert(3, 3), 3);
            assert_eq!(storage.indices.len(), 1);
            assert_eq!(*storage.get(3), 3);
            assert_eq!(*storage.get_mut(3), 3);
            assert_eq!(*storage.insert(1, 1), 1);
            assert_eq!(storage.indices.len(), 3);
            assert_eq!(storage.indices.get_unchecked_mut(0).assume_init(), 1);
            assert_eq!(storage.indices.get_unchecked_mut(2).assume_init(), 0);
            assert_eq!(*storage.get(1), 1);
            assert_eq!(*storage.get(3), 3);
            assert_eq!(*storage.insert(0, 0), 0);
            assert_eq!(storage.indices.len(), 4);
            assert_eq!(*storage.get(1), 1);
            assert_eq!(*storage.get(3), 3);
            assert_eq!(*storage.get(0), 0);

            let mut storage = DenseStorage::<i32>::default();
            storage.insert(4, 4);
            storage.insert(3, 3);
            storage.insert(2, 2);
            storage.insert(8, 8);
            storage.insert(6, 6);
            assert_eq!(storage.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(storage.data_id, vec![4, 3, 2, 8, 6]);
            assert_eq!(storage.base_id, Some(2));
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
            storage.insert(4, 4);
            storage.insert(3, 3);
            storage.insert(2, 2);
            storage.insert(8, 8);
            storage.insert(6, 6);
            assert_eq!(storage.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(storage.data_id, vec![4, 3, 2, 8, 6]);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[1].assume_init(), 1);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 4);
            assert_eq!(storage.indices[6].assume_init(), 3);

            storage.remove(3);
            assert_eq!(storage.data, vec![4, 6, 2, 8]);
            assert_eq!(storage.data_id, vec![4, 6, 2, 8]);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 1);
            assert_eq!(storage.indices[6].assume_init(), 3);

            storage.remove(8);
            assert_eq!(storage.data, vec![4, 6, 2]);
            assert_eq!(storage.data_id, vec![4, 6, 2]);
            assert_eq!(storage.base_id, Some(2));
            assert_eq!(storage.indices[0].assume_init(), 2);
            assert_eq!(storage.indices[2].assume_init(), 0);
            assert_eq!(storage.indices[4].assume_init(), 1);
        }
    }
}
