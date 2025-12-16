#[derive(Clone, Copy)]
pub struct BitIterator {
    value: u32,
    i: u8,
}

impl Iterator for BitIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 32 {
            None
        } else {
            self.i += 1;
            let bit = self.value & 1 != 0;
            self.value >>= 1;
            Some(bit)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (32-self.i as usize,Some(32-self.i as usize));
    }
}

impl DoubleEndedIterator for BitIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.i >= 32 {
            None
        } else {
            let index = 63 - self.i;
            let bit = (self.value >> index) & 1 != 0;
            self.i += 1;
            Some(bit)
        }
    }
}

impl ExactSizeIterator for BitIterator {}

pub trait BitIterable {
    /// Iter bits, from less significant to most significant, low index to high index.
    fn iter_bits(self) -> BitIterator;
}

impl BitIterable for u32 {
    fn iter_bits(self) -> BitIterator {
        BitIterator { value: self, i: 0 }
    }
}

impl BitIterable for &u32 {
    fn iter_bits(self) -> BitIterator {
        BitIterator { value: *self, i: 0 }
    }
}

/*
impl<T:IntoIterator<Item = u32>> BitIterable for T {
    fn iter_bits(self) -> impl Iterator<Item = bool> {
        self.into_iter().flat_map(|x| BitIterator {value:x,i:0})
    }
}*/
