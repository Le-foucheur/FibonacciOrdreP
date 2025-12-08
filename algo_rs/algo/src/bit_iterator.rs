use core::ops::Deref;

pub struct BitIterator{
    value: u32,
    i : u8,
}

impl Iterator for BitIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 32 {
            None
        } else {
            self.i += 1;
            let bit = self.value&1 != 0;
            self.value >>= 1;
            Some(bit)
        }
    }
}

pub trait BitIterable
{
    /// Iter bits, from less significant to most significant, low index to high index.
    fn iter_bits(self) -> impl Iterator<Item = bool>;
}

/*
impl BitIterable for u32 {
    fn iter_bits(self) -> impl Iterator<Item = bool> {
        BitIterator {
            value: self,
            i: 0,
        }
    }
}

impl BitIterable for &u32 {
    fn iter_bits(self) -> impl Iterator<Item = bool> {
        BitIterator {
            value: self,
            i: 0
        }
    }
}*/

impl<T:IntoIterator<Item : Deref<Target = u32>>> BitIterable for T {
    fn iter_bits(self) -> impl Iterator<Item = bool> {
        self.into_iter().flat_map(|x| BitIterator {value:*x,i:0})
    }
}
