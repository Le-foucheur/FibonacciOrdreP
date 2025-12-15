use crate::algo::Slice;

pub struct BitIterator<'a> {
    slice: Slice<'a, u32>,
    value: u32,
    i: u32,
}

impl<'a> Iterator for BitIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 32 {
            self.value = self.slice[0];
            self.slice.start +=1;
            self.i = 0;
        }
        self.i += 1;
        let bit = self.value & 1;
        self.value >>= 1;
        Some(bit)
    }
}

pub trait BitIterable<'a> {
    /// Iter bits, from less significant to most significant, low index to high index.
    fn iter_bits(self) -> BitIterator<'a>;
}

impl<'a> BitIterable<'a> for Slice<'a,u32> {
    fn iter_bits(self) -> BitIterator<'a> {
        BitIterator { slice:self,value: 0, i: 32 }
    }
}
