use std::alloc::{Allocator, Global, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use crate::collections::ring_vec::{IterFromCursor, RingCursor, RingVec};

pub struct EventChannel<E> {
    #[cfg(debug_assertions)]
    id: u64,
    readers: Vec<WeakReader>,
    events: RingVec<E>,
}

impl<E> EventChannel<E> {
    pub fn register(&mut self) -> ReaderHandle {
        let reader = Box::new(Reader {
            weak_count: AtomicU8::new(2),
            cursor: self.events.end_cursor(),
        });
        let reader = NonNull::from(Box::leak(reader));
        self.readers.push(WeakReader(reader));
        ReaderHandle {
            reader,
            #[cfg(debug_assertions)]
            channel_id: self.id,
        }
    }

    pub fn push(&mut self, e: E) {
        self.clean_zero_counted_reader();
        if self.readers.is_empty() {
            return;
        }
        self.events.push_back(e);
    }

    pub fn read(&self, reader: &mut ReaderHandle) -> IterFromCursor<'_, E> {
        #[cfg(debug_assertions)]
        assert_eq!(self.id, reader.channel_id);

        let cursor = &mut unsafe { reader.reader.as_mut() }.cursor;
        let iter = self.events.iter_from_cursor(*cursor).unwrap();
        *cursor = self.events.end_cursor();
        iter
    }

    pub fn read_any(&self, reader: &mut ReaderHandle) -> bool {
        let end = self.events.end_cursor();
        let cursor = &mut unsafe { reader.reader.as_mut() }.cursor;
        let any = *cursor < end;
        *cursor = end;
        any
    }

    fn clean_zero_counted_reader(&mut self) {
        let mut first: Option<RingCursor> = None;
        let mut i = 0;
        loop {
            if i < self.readers.len() {
                let reader = unsafe { self.readers[i].0.as_ref() };
                if reader.weak_count.load(Ordering::Acquire) > 1 {
                    let cursor = reader.cursor;
                    if let Some(f) = &first {
                        if cursor < *f {
                            first = Some(cursor)
                        }
                    } else {
                        first = Some(cursor)
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
            self.events.remove_to_cursor(first);
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

pub struct ReaderHandle {
    #[cfg(debug_assertions)]
    channel_id: u64,
    reader: NonNull<Reader>,
}

struct WeakReader(NonNull<Reader>);

struct Reader {
    weak_count: AtomicU8,
    cursor: RingCursor,
}

unsafe impl Send for ReaderHandle {}

impl Drop for ReaderHandle {
    fn drop(&mut self) {
        WeakReader(self.reader);
    }
}

unsafe impl Send for WeakReader {}

impl Drop for WeakReader {
    fn drop(&mut self) {
        unsafe {
            let reader = self.0.as_ref();
            if reader.weak_count.fetch_sub(1, Ordering::Release) == 1 {
                // Ensure that all access to the reader is visible
                // when the memory is about to be released
                reader.weak_count.load(Ordering::Acquire);

                Global.deallocate(self.0.cast(), Layout::for_value_raw(self.0.as_ptr()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_channel::EventChannel;

    #[test]
    fn drop_works() {
        let mut channel = EventChannel::default();
        let reader = channel.register();
        channel.push(());

        drop(reader);

        channel.push(());
        drop(channel);
    }
}
