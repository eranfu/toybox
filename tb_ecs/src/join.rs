use hibitset::{BitIter, BitSet, BitSetLike};

use tb_core::Id;
use tb_storage::StorageItems;

use crate::component::{ReadComponents, WriteComponents};
use crate::system::ReadOrder;
use crate::Component;

trait Join: Sized {
    type BitSet: BitSetLike;
    type Component;
    type Components;
    fn join(self) -> JoinIterator<Self> {
        let (mask, components) = self.open();
        JoinIterator {
            mask_iter: mask.iter(),
            components,
        }
    }
    fn open(self) -> (Self::BitSet, Self::Components);
    fn get(components: &mut Self::Components, id: Id) -> Self::Component;
}

struct JoinIterator<J: Join> {
    mask_iter: BitIter<J::BitSet>,
    components: J::Components,
}

impl<J: Join> Iterator for JoinIterator<J> {
    type Item = J::Component;

    fn next(&mut self) -> Option<Self::Item> {
        self.mask_iter
            .next()
            .map(|id| J::get(&mut self.components, id.into()))
    }
}

impl<'r, C: Component, R: ReadOrder> Join for &'r ReadComponents<'r, C, R> {
    type BitSet = &'r BitSet;
    type Component = &'r C;
    type Components = &'r C::Storage;

    fn open(self) -> (Self::BitSet, Self::Components) {
        self.components.storage.open()
    }

    fn get(components: &mut Self::Components, id: Id) -> Self::Component {
        unsafe { components.get(id) }
    }
}

impl<'r, C: Component> Join for &'r mut WriteComponents<'r, C> {
    type BitSet = &'r BitSet;
    type Component = &'r mut C;
    type Components = &'r mut C::Storage;

    fn open(self) -> (Self::BitSet, Self::Components) {
        self.components.storage.open_mut()
    }

    fn get(components: &mut Self::Components, id: Id) -> Self::Component {
        let components: *mut Self::Components = components as *mut Self::Components;
        unsafe { (*components).get_mut(id) }
    }
}
