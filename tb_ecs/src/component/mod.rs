use hibitset::BitSet;

mod storage;

pub trait Component: 'static + Sized {
    type Storage: Components<Self>;
}

pub trait Components<T> {}

pub(crate) struct ComponentsWithMask<C: Component> {
    mask: BitSet,
    components: C::Storage,
}