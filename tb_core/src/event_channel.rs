use std::ops::Deref;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock, Weak};

use errors::*;

use crate::collections::ring_vec::{IterFromCursor, RingCursor, RingVec};

mod errors {
    pub use crate::error::*;

    error_chain! {
        errors {
            ReaderChannelIdNotMatched
            IncorrectCursor
        }
    }
}

pub struct EventChannel<E> {
    #[cfg(debug_assertions)]
    id: u64,
    readers: Vec<WeakReader>,
    events: RingVec<E>,
}

impl<E> EventChannel<E> {
    pub fn register(&mut self) -> ReaderHandle {
        let reader = Arc::new(RwLock::new(Reader {
            cursor: self.events.end_cursor(),
        }));
        self.readers.push(WeakReader(Arc::downgrade(&reader)));
        ReaderHandle {
            reader,
            #[cfg(debug_assertions)]
            channel_id: self.id,
        }
    }

    pub fn push(&mut self, e: E) {
        self.clean_zero_counted_reader();
        self.events.push_back(e);
    }

    pub fn read(&self, reader: &ReaderHandle) -> IterFromCursor<'_, E> {
        #[cfg(debug_assertions)]
        assert_eq!(self.id, reader.channel_id);

        let mut reader = reader.reader.deref().write().unwrap();
        let iter = self.events.iter_from_cursor(reader.cursor).unwrap();
        reader.cursor = self.events.end_cursor();
        iter
    }

    pub fn read_any(&self, reader: &ReaderHandle) -> bool {
        let end = self.events.end_cursor();
        let mut reader = reader.reader.deref().write().unwrap();
        let any = reader.cursor < end;
        reader.cursor = end;
        any
    }

    fn clean_zero_counted_reader(&mut self) {
        let mut first: Option<Reader> = None;
        let mut i = 0;
        loop {
            if i < self.readers.len() {
                let reader = &self.readers[i];
                if let Some(reader) = reader.0.upgrade() {
                    let reader = reader.deref().read().unwrap();
                    if let Some(f) = &first {
                        if *reader < *f {
                            first = Some(*reader)
                        }
                    } else {
                        first = Some(*reader)
                    }
                    i += 1;
                } else {
                    self.readers.swap_remove(i);
                }
            } else {
                break;
            }
        }

        if let Some(first) = first {
            self.events.remove_to_cursor(first.cursor);
        } else {
            self.events.clear();
        }
    }
}

impl<E> Default for EventChannel<E> {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self {
            #[cfg(debug_assertions)]
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            readers: Default::default(),
            events: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct ReaderHandle {
    #[cfg(debug_assertions)]
    channel_id: u64,
    reader: Arc<RwLock<Reader>>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct Reader {
    cursor: RingCursor,
}

struct WeakReader(Weak<RwLock<Reader>>);
