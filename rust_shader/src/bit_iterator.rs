pub struct BitIterator {
    value: u32,
    i: u32,
}

impl Iterator for BitIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 32 {
            None
        } else {
            self.i += 1;
            let bit = self.value & 1;
            self.value >>= 1;
            Some(bit)
        }
    }
}

pub trait BitIterable {
    /// Iter bits, from less significant to most significant, low index to high index.
    fn iter_bits(self) -> BitIterator;
}

impl BitIterable for u32 {
    fn iter_bits(self) -> BitIterator {
        BitIterator { value: self, i: 0 }
    }
}
