extern crate libc;

use gmp_mpfr_sys::gmp::mpz_t;
use libc::c_uchar;
use std::{borrow::BorrowMut, cmp::min};
#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn fibo_mod2(p: isize, n: *mut mpz_t) -> *mut c_uchar;
    fn arr_getb(array: *const c_uchar, index: isize) -> bool;
}

pub struct FiboFastManager {
    pub p: u64,
    pub sequences: Vec<FiboFastSequence>,
}

impl FiboFastManager {
    pub fn new() -> FiboFastManager {
        FiboFastManager {
            p: 1,
            sequences: vec![FiboFastSequence::new(1)],
        }
    }

    pub fn generate(&mut self, p: u64, n: u64, mpz_start: mpz_t) -> Vec<bool> {
        if self.p < p {
            // Extend the sequences
            for i in self.p..p {
                self.sequences.push(FiboFastSequence::new(i + 1));
            }
            self.p = p;
        }
        // Generate the sequence
        self.sequences[p as usize - 1].generate(n, mpz_start)
    }
}

#[derive(Clone)]
pub struct FiboFastSequence {
    pub p: u64,
}

impl FiboFastSequence {
    pub fn new(p: u64) -> FiboFastSequence {
        FiboFastSequence { p }
    }

    pub fn generate(&mut self, n: u64, mut mpz_start: mpz_t) -> Vec<bool> {
        if self.p == 1 {
            return vec![false; n as usize];
        }

        let c_buf: *mut c_uchar =
            unsafe { fibo_mod2((self.p - 1).try_into().unwrap(), mpz_start.borrow_mut()) };
        // Use arr_getb to get the result
        let mut result = vec![false; n as usize];

        for i in 0..min(self.p + 1, n) {
            result[i as usize] = unsafe { arr_getb(c_buf, (self.p - i).try_into().unwrap()) };
        }
        // If the sequence is too short, extend it
        for i in (self.p + 1) as usize..n as usize {
            result[i] = result[i - 1] ^ result[i - self.p as usize];
        }
        result
    }
}
