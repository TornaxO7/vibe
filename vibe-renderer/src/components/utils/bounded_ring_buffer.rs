const DEFAULT_BLOCK_AMOUNT: usize = 1024;

pub struct Contigious<'a, T> {
    pub head: &'a [T],
    pub tail: &'a [T],
}

/// A ring buffer which can only `push_back()` and `pop_front()` and has a static
/// size.
#[derive(Debug)]
pub struct BoundedRingBuffer<T> {
    buffer: Box<[T]>,

    // <- head     tail ->
    // [.................]
    //
    // head: Contains the (inclusive) index where the longest stored value is
    // tail: Contains the (inclusive) index where the next value can be stored without overlapping
    head: usize,
    tail: usize,

    is_full: bool,
}

impl<T: Default> Default for BoundedRingBuffer<T> {
    fn default() -> Self {
        Self::new(DEFAULT_BLOCK_AMOUNT)
    }
}

impl<T: Default> BoundedRingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Code assumes a capacity > 0");

        let buffer: Box<[T]> = {
            let mut buffer = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                buffer.push(T::default());
            }

            buffer.into_boxed_slice()
        };

        Self {
            buffer,

            head: 0,
            tail: 0,

            is_full: false,
        }
    }

    /// Returns `true` if the value could be added.
    pub fn push_back(&mut self, value: T) -> bool {
        if self.is_full {
            return false;
        }

        self.buffer[self.tail] = value;
        self.inc_tail();

        if self.tail == self.head {
            self.is_full = true;
        }

        true
    }

    /// Increments the `head` pointer if it's not empty.
    pub fn pop_front(&mut self) {
        if !self.is_empty() {
            self.is_full = false;

            self.inc_head();

            if self.tail_will_wrap() {
                self.tail = 0;
            }
        }
    }

    /// Returns the amount of values which are currently stored.
    pub fn len(&self) -> usize {
        if self.is_full {
            self.buffer.len()
        } else if self.head == self.tail {
            0
        } else if self.head < self.tail {
            self.tail - self.head
        } else {
            (self.tail + 1) + (self.buffer.len() - self.head)
        }
    }

    #[allow(unused)]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    fn is_empty(&self) -> bool {
        !self.is_full && (self.head == self.tail)
    }

    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    fn tail_will_wrap(&self) -> bool {
        self.tail == self.capacity() - 1
    }

    /// Increment the `tail` index.
    fn inc_tail(&mut self) {
        self.tail = self.peek_inc_tail();
    }

    /// Returns the next index value of `tail` if it would get incremented.
    fn peek_inc_tail(&self) -> usize {
        if self.tail_will_wrap() {
            0
        } else {
            self.tail + 1
        }
    }

    /// Increment the `head` index.
    fn inc_head(&mut self) {
        self.head = self.peek_inc_head();
    }

    /// Returns the next index value of `head` if it would get incremented.
    fn peek_inc_head(&self) -> usize {
        if self.head_will_wrap() {
            0
        } else {
            self.head + 1
        }
    }

    /// Returns `true` if `head` will be at the beginning of the internal buffer
    /// at the next incrementation.
    fn head_will_wrap(&self) -> bool {
        self.head == self.capacity() - 1
    }

    /// Returns two slices which, if you put `tail` after `head`, would
    /// make the contigious array.
    pub fn contigious(&self) -> Contigious<'_, T> {
        if self.head <= self.tail {
            Contigious {
                head: &self.buffer[self.head..self.tail],
                tail: &[],
            }
        } else {
            Contigious {
                head: &self.buffer[self.head..],
                tail: &self.buffer[0..self.tail],
            }
        }
    }

    pub fn pop_while(&mut self, cond: impl Fn(&T) -> bool) {
        loop {
            if self.is_empty() {
                break;
            }

            let value = &self.buffer[self.head];
            if cond(value) {
                self.pop_front();
            } else {
                break;
            }
        }
    }
}

pub struct BoundedRingBufferIterator<T> {
    index: usize,

    inner_data: Box<[T]>,
    end_index: usize,

    in_beginning: bool,
}

impl<T: Default> Iterator for BoundedRingBufferIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end_index {
            if self.in_beginning {
                self.in_beginning = false;

                let mut value = T::default();
                std::mem::swap(&mut self.inner_data[self.index], &mut value);

                self.index += 1;
                if self.index == self.inner_data.len() {
                    self.index = 0;
                }

                return Some(value);
            } else {
                return None;
            }
        }

        let mut value = T::default();
        std::mem::swap(&mut self.inner_data[self.index], &mut value);

        self.index += 1;
        if self.index == self.inner_data.len() {
            self.index = 0;
        }

        Some(value)
    }
}

impl<T: Default> IntoIterator for BoundedRingBuffer<T> {
    type Item = T;
    type IntoIter = BoundedRingBufferIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        BoundedRingBufferIterator {
            index: self.head,
            end_index: self.tail,
            in_beginning: !self.is_empty(),
            inner_data: self.buffer,
        }
    }
}

pub struct Iter<'a, T> {
    buffer: &'a BoundedRingBuffer<T>,
    idx: usize,
    in_beginning: bool,
}

impl<'a, T: Default> Iter<'a, T> {
    pub fn new(buffer: &'a BoundedRingBuffer<T>) -> Self {
        Self {
            buffer,
            idx: buffer.head,
            in_beginning: !buffer.is_empty(),
        }
    }
}

impl<'a, T: Default> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.buffer.tail {
            if self.in_beginning {
                self.in_beginning = false;

                let idx = self.idx;

                self.idx += 1;
                if self.idx == self.buffer.capacity() {
                    self.idx = 0;
                }

                return self.buffer.buffer.get(idx);
            } else {
                return None;
            }
        }

        let idx = self.idx;
        self.idx += 1;
        if self.idx == self.buffer.capacity() {
            self.idx = 0;
        }

        self.buffer.buffer.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Replace the comments which should show the state of the array
    //       with assertions.

    #[test]
    fn empty_pop_front() {
        let mut buffer = BoundedRingBuffer::<u8>::new(1);
        buffer.pop_front();

        assert_eq!(buffer.head, 0);
        assert_eq!(buffer.tail, 0);
        assert_eq!(buffer.len(), 0);
        assert!(!buffer.is_full);
    }

    #[test]
    fn empty_push_back() {
        let mut buffer = BoundedRingBuffer::<u8>::new(1);
        assert!(buffer.push_back(69));

        assert_eq!(buffer.head, 0);
        assert_eq!(buffer.tail, 0);
        assert!(buffer.is_full);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.buffer[0], 69);
    }

    #[test]
    fn full_push_back() {
        let mut buffer = BoundedRingBuffer::<u8>::new(1);
        assert!(buffer.push_back(69));
        assert!(!buffer.push_back(42));

        assert_eq!(buffer.head, 0);
        assert_eq!(buffer.tail, 0);
        assert!(buffer.is_full);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.buffer[0], 69);
    }

    #[test]
    fn wrap_push_back() {
        let mut buffer = BoundedRingBuffer::<u8>::new(1);
        assert!(buffer.push_back(69));
        buffer.pop_front();

        // should be in the beginning again
        assert_eq!(buffer.head, 0);
        assert_eq!(buffer.tail, 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        assert!(buffer.push_back(42));
        assert_eq!(buffer.head, 0);
        assert_eq!(buffer.tail, 0);
        assert!(!buffer.is_empty());
        assert!(buffer.is_full);
        assert_eq!(buffer.buffer[0], 42);
    }

    #[test]
    fn wrapped_state() {
        let mut buffer = BoundedRingBuffer::<u8>::new(2);
        assert!(buffer.push_back(69));
        assert!(buffer.push_back(69));
        buffer.pop_front();

        assert_eq!(buffer.head, 1);
        assert_eq!(buffer.tail, 0);
        assert!(buffer.push_back(42));

        assert_eq!(buffer.head, 1);
        assert_eq!(buffer.tail, 1);
        assert!(buffer.is_full);
        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.buffer.as_ref(), &[42, 69]);
    }

    mod iterator {
        use super::*;

        #[test]
        fn empty() {
            let buffer = BoundedRingBuffer::<u8>::new(1);
            let mut iterator = buffer.into_iter();
            assert!(iterator.next().is_none());
        }

        #[test]
        fn full() {
            let mut buffer = BoundedRingBuffer::<u8>::new(1);
            assert!(buffer.push_back(69));

            let mut iterator = buffer.into_iter();
            assert_eq!(iterator.next(), Some(69));
            assert!(iterator.next().is_none());
        }

        #[test]
        fn wrapped() {
            let mut buffer = BoundedRingBuffer::<u8>::new(3);

            // head   tail
            //  |      |
            // [0, 1, 2]
            for i in 0..3 {
                assert!(buffer.push_back(i));
            }

            //      head tail
            //        |  |
            // [?, ?, 2]
            buffer.pop_front();
            buffer.pop_front();

            //   tail head
            //     |  |
            // [0, 1, 2]
            assert!(buffer.push_back(0));
            assert!(buffer.push_back(1));

            let mut iterator = buffer.into_iter();
            assert_eq!(iterator.next(), Some(2));
            assert_eq!(iterator.next(), Some(0));
            assert_eq!(iterator.next(), Some(1));
            assert!(iterator.next().is_none());
        }
    }

    mod iter {
        use super::*;

        #[test]
        fn empty() {
            let buffer = BoundedRingBuffer::<u8>::new(1);
            assert!(buffer.iter().next().is_none());
        }

        #[test]
        fn full() {
            let mut buffer = BoundedRingBuffer::<u8>::new(1);
            assert!(buffer.push_back(69));

            let mut i = buffer.iter();
            assert_eq!(i.next(), Some(&69));
            assert!(i.next().is_none());
        }

        #[test]
        fn wrapped() {
            let mut buffer = BoundedRingBuffer::<u8>::new(3);

            // head   tail
            //  |      |
            // [0, 1, 2]
            for i in 0..3 {
                assert!(buffer.push_back(i));
            }

            //      head tail
            //        |  |
            // [?, ?, 2]
            buffer.pop_front();
            buffer.pop_front();

            //   tail head
            //     |  |
            // [0, 1, 2]
            assert!(buffer.push_back(0));
            assert!(buffer.push_back(1));

            let mut iterator = buffer.iter();
            assert_eq!(iterator.next(), Some(&2));
            assert_eq!(iterator.next(), Some(&0));
            assert_eq!(iterator.next(), Some(&1));
            assert!(iterator.next().is_none());
        }
    }

    mod contigious {
        use super::*;

        #[test]
        fn empty() {
            let buffer = BoundedRingBuffer::<u8>::new(1);
            let c = buffer.contigious();
            assert!(c.head.is_empty());
            assert!(c.tail.is_empty());
        }

        #[test]
        fn one_value() {
            let mut buffer = BoundedRingBuffer::<u8>::new(1);
            // [69]
            buffer.push_back(69);
            let c = buffer.contigious();
            assert_eq!(c.head, &[69]);
            assert!(c.tail.is_empty());
        }

        #[test]
        fn wrapped() {
            let mut buffer = BoundedRingBuffer::<u8>::new(3);

            // [0, 1, 2]
            for i in 0..3 {
                buffer.push_back(i);
            }
            assert_eq!(buffer.buffer.as_ref(), &[0, 1, 2]);

            // [?, ?, 2]
            for _ in 0..2 {
                buffer.pop_front();
            }
            assert_eq!(buffer.buffer[2], 2);

            // [1, ?, 2]
            buffer.push_back(1);
            assert_eq!(buffer.buffer[0], 1);
            assert_eq!(buffer.buffer[2], 2);

            let c = buffer.contigious();
            assert_eq!(c.head, &[2]);
            assert_eq!(c.tail, &[1]);
        }
    }

    mod pop_while {
        use super::*;

        #[test]
        fn empty() {
            let mut buffer = BoundedRingBuffer::<u8>::new(1);
            buffer.pop_while(|v| *v == 7);

            assert_eq!(buffer.head, 0);
            assert_eq!(buffer.tail, 0);
            assert!(buffer.is_empty());
        }

        #[test]
        fn vec_like_filled_pop_all() {
            let mut buffer = BoundedRingBuffer::<u8>::new(5);

            // [1, 2, 3, 4, 5]
            for i in 0..5 {
                buffer.push_back(i);
            }

            buffer.pop_while(|v| *v < 6);

            assert!(buffer.is_empty());
            assert_eq!(buffer.head, 0);
            assert_eq!(buffer.tail, 0);
        }

        #[test]
        fn wrapped_filled_pop_all() {
            let mut buffer = BoundedRingBuffer::<u8>::new(3);

            for i in 0..3 {
                buffer.push_back(i);
            }
            assert_eq!(buffer.buffer.as_ref(), &[0, 1, 2]);

            for _ in 0..2 {
                buffer.pop_front();
            }
            assert_eq!(buffer.buffer[2], 2);

            // [1, ?, 2]
            buffer.push_back(1);
            assert_eq!(buffer.buffer[0], 1);
            assert_eq!(buffer.buffer[2], 2);

            buffer.pop_while(|v| *v < 3);

            assert!(buffer.is_empty());
            assert_eq!(buffer.head, 1);
            assert_eq!(buffer.tail, 1);
        }

        #[test]
        fn wrapped_filled_pop_one() {
            let mut buffer = BoundedRingBuffer::<u8>::new(3);

            for i in 0..3 {
                buffer.push_back(i);
            }
            assert_eq!(buffer.buffer.as_ref(), &[0, 1, 2]);

            for _ in 0..2 {
                buffer.pop_front();
            }
            assert_eq!(buffer.buffer[2], 2);

            // [1, ?, 2]
            buffer.push_back(1);
            assert_eq!(buffer.buffer[0], 1);
            assert_eq!(buffer.buffer[2], 2);

            buffer.pop_while(|v| *v != 1);

            assert_eq!(buffer.head, 0);
            assert_eq!(buffer.tail, 1);
            assert_eq!(buffer.len(), 1);
            assert_eq!(buffer.buffer[0], 1);
        }
    }
}
