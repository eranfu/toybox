use tb_storage::Storage;

pub trait Component: 'static + Sized {
    type Storage: Storage<Data = Self>;
}
