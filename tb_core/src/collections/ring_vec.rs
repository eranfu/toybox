use std::mem::MaybeUninit;
use std::ptr;

pub struct RingVec<T> {
    buf: Vec<MaybeUninit<T>>,
    start: usize,
    len: usize,
}

impl<T> RingVec<T> {
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
}

impl<T> Default for RingVec<T> {
    fn default() -> Self {
        Self {
            buf: vec![],
            start: 0,
            len: 0,
        }
    }
}

impl<T> Drop for RingVec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                let i = (self.start + i) % self.buf.len();
                self.buf[i].assume_init_drop();
            }
            self.buf.set_len(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::RingVec;

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

    #[test]
    fn it_works() {
        const TEST_NUM: i32 = 10000;
        let mut ring = RingVec::default();
        let mut pop_cursor = 0;
        let mut push_cursor = 0;
        while pop_cursor < TEST_NUM {
            if rand::random() {
                if push_cursor < TEST_NUM {
                    ring.push_back(Info(push_cursor));
                    push_cursor += 1;
                }
            } else if pop_cursor < push_cursor {
                assert_eq!(ring.pop_front(), Some(Info(pop_cursor)));
                pop_cursor += 1;
            } else {
                assert_eq!(ring.pop_front(), None);
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
