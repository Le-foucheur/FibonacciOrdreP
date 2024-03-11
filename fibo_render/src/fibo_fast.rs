extern crate libc;

use libc::c_char;
use libc::c_uchar;
use std::ffi::CStr;
use std::time::Duration;
use std::time::Instant;

#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn rust_fibo_mod2(p: isize, n: *const c_char) -> *mut c_uchar;
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

    pub fn generate(&mut self, p: u64, n: u64, start: u64) -> Vec<bool> {
        if self.p < p {
            // Extend the sequences
            for i in self.p..p {
                self.sequences.push(FiboFastSequence::new(i + 1));
            }
            self.p = p;
        }
        // Generate the sequence
        self.sequences[p as usize - 1].generate(n, start)
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

    pub fn generate(&mut self, n: u64, start: u64) -> Vec<bool> {
        if self.p == 1 {
            return vec![false; (n - start + self.p) as usize];
        }
        // convert n to base 10 string
        let temp = format!("{}\0", start);
        let temp = temp.as_bytes();
        let n_uchar = CStr::from_bytes_with_nul(temp).unwrap();
        let k = n_uchar.as_ptr();
        let now = Instant::now();
        let c_buf: *mut c_uchar = unsafe { rust_fibo_mod2((self.p - 1).try_into().unwrap(), k) };
        // print!("Time1: {}\n", now.elapsed().as_micros());
        let now = Instant::now();
        // Use arr_getb to get the result
        let mut result = vec![false; (self.p+1) as usize];
        for i in 0..self.p + 1 {
            result[i as usize] = unsafe { arr_getb(c_buf, (self.p - i).try_into().unwrap()) };
        }
        // print!("Time2: {}\n", now.elapsed().as_micros());
        let now = Instant::now();
        // If the sequence is too short, extend it
        let size = result.len();
        result.resize((n - start + self.p) as usize, false);
        for i in size..(n - start + self.p) as usize {
            result[i] = result[i - 1] ^ result[i - self.p as usize];
        }
        // print!("Time3: {}\n", now.elapsed().as_micros());
        result[self.p as usize..].to_vec()
    }
}
