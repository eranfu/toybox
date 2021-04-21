use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::collections::ring_vec::{IterFromCursor, RingCursor, RingVec};

pub struct EventChannel<E> {
    readers: BinaryHeap<Reverse<WeakReader>>,
    events: RingVec<E>,
}

impl<E> EventChannel<E> {
    pub fn register(&mut self) -> ReaderHandle {
        let reader = Rc::new(RefCell::new(Reader {
            cursor: self.events.end_cursor(),
        }));
        self.readers
            .push(Reverse(WeakReader(Rc::downgrade(&reader))));
        ReaderHandle(reader)
    }

    pub fn push(&mut self, e: E) {
        self.events.push_back(e);
    }

    pub fn read(&self, reader: ReaderHandle) -> IterFromCursor<'_, E> {
        let mut reader = reader.0.deref().borrow_mut();
        let iter = self.events.iter_from_cursor(&reader.cursor).unwrap();
        reader.cursor = self.events.end_cursor();
        iter
    }
}

impl<E> Default for EventChannel<E> {
    fn default() -> Self {
        Self {
            readers: BinaryHeap::default(),
            events: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct ReaderHandle(Rc<RefCell<Reader>>);

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Reader {
    cursor: RingCursor,
}

struct WeakReader(Weak<RefCell<Reader>>);

impl PartialEq for WeakReader {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.upgrade(), other.0.upgrade()) {
            (Some(s), Some(o)) => s.deref().eq(o.deref()),
            (None, None) => true,
            _ => false,
        }
    }
}

impl Eq for WeakReader {}

impl PartialOrd for WeakReader {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeakReader {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.upgrade(), other.0.upgrade()) {
            (Some(s), Some(o)) => s.deref().cmp(o.deref()),
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
        }
    }
}
