use hibitset::BitSet;

use tb_storage::Storage;

pub trait Component: 'static + Sized {
    type Storage: Storage<Self>;
}

pub struct Components<C: Component> {
    mask: BitSet,
    components: C::Storage,
}
