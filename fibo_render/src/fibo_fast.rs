use algo::bit_iterator::BitIterable;
use num::{bigint::Sign, BigInt};
/*extern "C" {
    fn fibo_mod2_initialization(p: isize) -> *mut c_uchar;
    fn fibo_mod2(p: isize, n: *mut mpz_t) -> *mut c_uchar;
    fn arr_getb(array: *const c_uchar, index: isize) -> bool;
}*/

pub struct FiboFastManager {
    scratch1: Vec<u32>,
    scratch2: Vec<u32>,
    pub p: usize,
    pub sequences: Vec<FiboFastSequence>,
}

impl FiboFastManager {
    pub fn new() -> FiboFastManager {
        FiboFastManager {
            scratch1: Vec::new(),
            scratch2: Vec::new(),
            p: 1,
            sequences: vec![FiboFastSequence::new(1)],
        }
    }

    pub fn generate(&mut self, p: usize, n: usize, end: BigInt) -> impl Iterator<Item = bool> + use<'_> {
        if self.p < p {
            // Extend the sequences
            for i in self.p..p {
                self.sequences.push(FiboFastSequence::new(i + 1));
            }
            self.p = p;
        }
        // Generate the sequence
        self.sequences[p - 1].generate(&mut self.scratch1, &mut self.scratch2, n, end)
    }
}

#[derive(Clone)]
pub struct FiboFastSequence {
    pub p: usize,
    pub saved: Vec<u32>,
}

impl FiboFastSequence {
    pub fn new(p: usize) -> FiboFastSequence {
        FiboFastSequence { p, saved: vec![] }
    }
    pub fn generate(
        &mut self,
        scratch1: &mut Vec<u32>,
        scratch2: &mut Vec<u32>,
        n: usize,
        end: BigInt,
    ) -> impl Iterator<Item = bool> + use<'_> {
        /*
        // Manage this special case lonely, because it crash the C library
        if self.p == 1 {
            let mut result = vec![false; n as usize];
            result[0] = true;
            return result;
        }
        // Call the C library
        let c_buf: *mut c_uchar =
            unsafe { fibo_mod2((self.p - 1).try_into().unwrap(), mpz_end.borrow_mut()) };


        // Load the result in the end of the array
        // Start variable is useful when self.p is bigger than the asked size n
        let start = if self.p + 1 < n { 0 } else { self.p + 1 - n };
        for i in start..self.p + 1 {
            // Use arr_getb to get the result
            result[(n + i - self.p - 1) as usize] =
                unsafe { arr_getb(c_buf, (self.p - i).try_into().unwrap()) };
        }

        // If the sequence is too short, extend it from right to left
        if self.p < n {
            for i in 0..(n - self.p) as usize {
                result[(n - self.p) as usize - i - 1] =
                    result[n as usize - i - 1] ^ result[n as usize - i - 2]
            }
        }
        result
        */
        // Initialize the result array
        self.saved.resize(n.div_ceil(32), 0);
        let params = algo::setup(self.p);
        scratch1.resize(params.ranges_size, 0);
        scratch2.resize(params.ranges_size, 0);


        algo::calculator(
            scratch1,
            scratch2,
            &mut self.saved,
            params,
            end.iter_u32_digits().rev().flat_map(
                |limb| limb.iter_bits().rev()
            ),
            end.sign() == Sign::Minus,
        );

        self.saved.iter().copied().flat_map(|limb| limb.iter_bits()).take(n)
    }
}
