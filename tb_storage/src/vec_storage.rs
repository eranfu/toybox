use std::mem::MaybeUninit;
use std::ptr;

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

impl<D> Storage<D> for VecStorage<D> {
    fn clear(&mut self) {
        self.data.clear();
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        util::setup_base_id(&mut self.base_id, &mut self.data, id);
        let index = self.get_index(id);
        util::ensure_index(&mut self.data, index);
        self.data.get_unchecked_mut(index).as_mut_ptr().write(data);
        &mut *self.data.get_unchecked_mut(index).as_mut_ptr()
    }

    unsafe fn remove(&mut self, id: Id) {
        ptr::read(self.get(id));
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
