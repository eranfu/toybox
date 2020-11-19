use std::mem::MaybeUninit;

use tb_core::Id;

use crate::Storage;

pub struct DenseStorage<D> {
    data: Vec<D>,
    data_id: Vec<Id>,
    indices: Vec<MaybeUninit<usize>>,
}

impl<D> Storage<D> for DenseStorage<D> {
    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        if self.indices.len() <= id as usize {
            self.indices.reserve(id as usize + 1 - self.indices.len());
            self.indices.set_len((id as usize) + 1);
        }

        let index = self.data.len();
        self.indices
            .get_unchecked_mut(id as usize)
            .as_mut_ptr()
            .write(index);

        self.data.push(data);
        self.data_id.push(id);

        self.data.get_unchecked_mut(index)
    }

    unsafe fn get(&self, id: u32) -> &D {
        let index = self.indices.get_unchecked(id as usize).assume_init();
        self.data.get_unchecked(index)
    }

    unsafe fn get_mut(&mut self, id: u32) -> &mut D {
        let index = self.indices.get_unchecked(id as usize).assume_init();
        self.data.get_unchecked_mut(index)
    }
}
