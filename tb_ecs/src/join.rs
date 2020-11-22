use hibitset::{BitIter, BitSetAll};

use tb_core::Id;

trait Join: Sized {
    type Item;
    fn join(self) -> JoinIterator<Self>;
    fn get(&self, id: Id) -> Self::Item;
}

struct JoinIterator<J: Join> {
    mask_iter: BitIter<BitSetAll>,
    join: J,
}

impl<J: Join> Iterator for JoinIterator<J> {
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.mask_iter.next().map(|id| self.join.get(id))
    }
}
