use hibitset::{BitIter, BitSet, BitSetLike};

use tb_core::Id;
use tb_storage::Storage;

use crate::component::{RAWComponents, RBWComponents, WriteComponents};
use crate::Component;

trait Join: Sized {
    type Item;
    type BitSetLike: BitSetLike;
    fn join(self) -> JoinIterator<Self, Self::BitSetLike>;
    fn get(&self, id: Id) -> Self::Item;
}

struct JoinIterator<J, B>
where
    B: BitSetLike,
    J: Join<BitSetLike = B>,
{
    mask_iter: BitIter<B>,
    join: J,
}

trait ComponentsData<'r> {
    type Component: 'r;
    fn get(&'r self, id: Id) -> Self::Component;
    fn mask(&'r self) -> &'r BitSet;
}

impl<'r, C: Component> ComponentsData<'r> for &'r RBWComponents<'r, C> {
    type Component = &'r C;

    fn get(&self, id: Id) -> Self::Component {
        self.components().get(id)
    }

    fn mask(&self) -> &BitSet {
        self.components().mask()
    }
}

impl<'r, C: Component> ComponentsData<'r> for &'r mut WriteComponents<'r, C> {
    type Component = &'r mut C;

    fn get(&'r self, id: Id) -> Self::Component {
        self.components().get_mut(id)
    }

    fn mask(&'r self) -> &'r BitSet {
        self.components().mask()
    }
}

impl<'r, C: Component> ComponentsData<'r> for &'r RAWComponents<'r, C> {
    type Component = &'r C;

    fn get(&self, id: Id) -> Self::Component {
        self.components().get(id)
    }

    fn mask(&self) -> &BitSet {
        self.components().mask()
    }
}

impl<J, B> Iterator for JoinIterator<J, B>
where
    B: BitSetLike,
    J: Join<BitSetLike = B>,
{
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.mask_iter.next() {
            None => None,
            Some(id) => Some(self.join.get(id.into())),
        }
    }
}

impl<'r, D> Join for &'r D
where
    D: ComponentsData<'r>,
{
    type Item = D::Component;
    type BitSetLike = &'r BitSet;

    fn join(self) -> JoinIterator<Self, Self::BitSetLike> {
        JoinIterator {
            mask_iter: self.mask().iter(),
            join: self,
        }
    }

    fn get(&self, id: Id) -> Self::Item {
        ComponentsData::get(*self, id)
    }
}
