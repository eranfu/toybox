#![feature(maybe_uninit_ref)]

pub use dense_storage::DenseStorage;
use tb_core::Id;

pub trait Storage<D> {
    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D;
    unsafe fn get(&self, id: Id) -> &D;
    unsafe fn get_mut(&mut self, id: Id) -> &mut D;
}

mod dense_storage;
