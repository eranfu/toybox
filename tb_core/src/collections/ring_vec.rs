use std::hash::Hash;
use std::mem::MaybeUninit;
use std::ops::Index;
use std::ptr;

pub struct RingVec<T> {
    buf: Vec<MaybeUninit<T>>,
    start: usize,
    len: usize,
    pop_counter: u64,
}

impl<T> RingVec<T> {
    pub fn clear(&mut self) {
        self.remove_to_index(self.len);
    }

    pub fn iter_from_cursor(&self, from_cursor: RingCursor) -> Option<IterFromCursor<T>> {
        self.cursor_to_index(from_cursor)
            .map(|index| IterFromCursor::new(index, self))
    }

    pub fn remove_to_cursor(&mut self, to_cursor: RingCursor) {
        if let Some(index) = self.cursor_to_index(to_cursor) {
            self.remove_to_index(index);
        }
    }

    pub fn end_cursor(&self) -> RingCursor {
        RingCursor {
            cursor: self.pop_counter + self.len as u64,
        }
    }

    pub fn get_by_cursor(&self, cursor: RingCursor) -> Option<&T> {
        self.cursor_to_index(cursor)
            .and_then(|index| self.get_by_index(index))
    }

    pub fn push_back(&mut self, value: T) {
        self.reserve(1);
        unsafe {
            let end = self
                .buf
                .as_mut_ptr()
                .add((self.start + self.len) % self.buf.len());
            ptr::write(end, MaybeUninit::new(value));
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                let start = self.buf.as_ptr().add(self.start);
                self.start = (self.start + 1) % self.buf.len();
                self.len -= 1;
                self.pop_counter += 1;
                Some(ptr::read(start).assume_init())
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let new_buf_len = self.len + additional;
        let old_buf_len = self.buf.len();
        if new_buf_len > old_buf_len {
            let additional = (new_buf_len - old_buf_len).max(old_buf_len);
            self.buf.reserve(additional);
            unsafe {
                self.buf.set_len(self.buf.capacity());
            }
            if (self.start + self.len) > old_buf_len {
                let additional = self.buf.len() - old_buf_len;
                let tail_num = old_buf_len - self.start;
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.buf.as_ptr().add(self.start),
                        self.buf.as_mut_ptr().add(self.start + additional),
                        tail_num,
                    );
                }
                self.start += additional;
            }
        }
    }

    fn get_by_index(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            None
        } else {
            let index = (self.start + index) % self.buf.len();
            Some(unsafe { (self.buf[index]).assume_init_ref() })
        }
    }

    fn remove_to_index(&mut self, to_index: usize) {
        let to_index = to_index.min(self.len);
        if to_index == 0 {
            return;
        }
        for i in 0..to_index {
            let i = (self.start + i) % self.buf.len();
            unsafe {
                self.buf[i].assume_init_drop();
            }
        }
        self.start = (self.start + to_index) % self.buf.len();
        self.len -= to_index;
    }

    fn cursor_to_index(&self, cursor: RingCursor) -> Option<usize> {
        let cursor = cursor.cursor;
        if cursor < self.pop_counter {
            None
        } else {
            Some((cursor - self.pop_counter) as usize)
        }
    }
}

impl<T> Default for RingVec<T> {
    fn default() -> Self {
        Self {
            buf: vec![],
            start: 0,
            len: 0,
            pop_counter: 0,
        }
    }
}

impl<T> Drop for RingVec<T> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.buf.set_len(0);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RingCursor {
    cursor: u64,
}

impl<T> Index<RingCursor> for RingVec<T> {
    type Output = T;

    fn index(&self, index: RingCursor) -> &Self::Output {
        self.get_by_cursor(index).unwrap()
    }
}

pub struct IterFromCursor<'r, T> {
    cur: usize,
    ring_vec: &'r RingVec<T>,
}

impl<'r, T> IterFromCursor<'r, T> {
    fn new(from_index: usize, ring_vec: &'r RingVec<T>) -> Self {
        Self {
            cur: from_index,
            ring_vec,
        }
    }
}

impl<'r, T> Iterator for IterFromCursor<'r, T> {
    type Item = &'r T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ring_vec.get_by_index(self.cur) {
            None => None,
            cur @ Some(_) => {
                self.cur += 1;
                cur
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    use crate::collections::ring_vec::RingVec;

    static mut DROP_HISTORY: Vec<i32> = vec![];

    #[derive(Eq, PartialEq, Debug)]
    struct Info(i32);

    impl Drop for Info {
        fn drop(&mut self) {
            unsafe {
                DROP_HISTORY.push(self.0);
            }
        }
    }

    enum RandomOp {
        Push,
        Pop,
        GetCursor,
        CheckCursor,
    }

    impl Distribution<RandomOp> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandomOp {
            match rng.next_u32() % 4 {
                0 => RandomOp::Push,
                1 => RandomOp::Pop,
                2 => RandomOp::GetCursor,
                3 => RandomOp::CheckCursor,
                _ => {
                    unreachable!()
                }
            }
        }
    }

    #[test]
    fn it_works() {
        const TEST_NUM: i32 = 10000;
        let mut ring = RingVec::default();
        let mut pop_cursor = 0;
        let mut push_cursor = 0;
        while pop_cursor < TEST_NUM {
            match rand::random::<RandomOp>() {
                RandomOp::Push => {
                    if push_cursor < TEST_NUM {
                        ring.push_back(Info(push_cursor));
                        push_cursor += 1;
                    }
                }
                RandomOp::Pop => {
                    if pop_cursor < push_cursor {
                        assert_eq!(ring.pop_front(), Some(Info(pop_cursor)));
                        pop_cursor += 1;
                    } else {
                        assert_eq!(ring.pop_front(), None);
                    }
                }
                RandomOp::GetCursor => {}
                RandomOp::CheckCursor => {}
            }
        }
        assert_eq!(ring.pop_front(), None);
        for i in 0..TEST_NUM {
            ring.push_back(Info(i))
        }
        drop(ring);

        unsafe {
            let mut assert_drop = vec![];
            for x in 0..TEST_NUM {
                assert_drop.push(x);
                assert_drop.push(x);
            }
            for x in 0..TEST_NUM {
                assert_drop.push(x);
            }
            assert_eq!(DROP_HISTORY, assert_drop);
        }
    }
}
