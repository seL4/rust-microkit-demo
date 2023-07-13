#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RingBuffer<T, const SIZE: usize> {
    take_index: usize,
    put_index: usize,
    entries: [T; SIZE],
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn empty() -> Self {
        Self {
            take_index: 0,
            put_index: 0,
            entries: [T::default(); SIZE],
        }
    }
}

impl<T: Default + Copy, const SIZE: usize> core::iter::FromIterator<T> for RingBuffer<T, SIZE> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = Self::empty();

        for entry in iter {
            res.put(entry).expect("Too many elements in iterator-to-ringbuffer conversion");
        }

        res
    }
}

impl<T: Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn len(&self) -> usize {
        ((SIZE + self.put_index) - self.take_index) % SIZE
    }

    pub fn is_full(&self) -> bool {
        self.len() == SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn take(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let entry = self.entries[self.take_index].clone();
            self.take_index = (self.take_index + 1) % SIZE;

            Some(entry)
        }
    }

    pub fn put(&mut self, entry: T) -> Option<()> {
        if self.is_full() {
            None
        } else {
            self.entries[self.put_index] = entry;
            self.put_index = (self.put_index + 1) % SIZE;

            Some(())
        }
    }
}
