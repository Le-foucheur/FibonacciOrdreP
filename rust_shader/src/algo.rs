use crate::bit_iterator::BitIterable;
use core::{
    hint::{likely, unlikely},
    mem::swap,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy)]
pub struct Slice<'a, T> {
    buffer: *mut &'a mut [T],
    pub start: usize,
    len: usize,
}

impl<'a, T> IndexMut<usize> for Slice<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut (*self.buffer)[self.start + index] }
    }
}

impl<'a, T> Index<usize> for Slice<'a, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &(*self.buffer)[self.start + index] }
    }
}
impl<'a, T> Slice<'a, T> {
    pub unsafe fn new(buffer: *mut &'a mut [T], start: usize, len: usize) -> Self {
        Self { buffer, start, len }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn iter_val(self) -> SliceIterVal<'a, T> {
        SliceIterVal(self)
    }
    pub fn skip(self, len: usize) -> Self {
        Self {
            buffer: self.buffer,
            start: self.start + len,
            len: self.len - len,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SliceIterVal<'a, T>(Slice<'a, T>);
impl<'a, T: Copy + 'a> Iterator for SliceIterVal<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len == 0 {
            None
        } else {
            let next = unsafe { (*self.0.buffer)[self.0.start] };
            self.0.start += 1;
            self.0.len -= 1;
            Some(next)
        }
    }
}
/// turn abc, def in daebcf  (b will be shifted once more / a rightmost bit will stay there)
/// see https://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN for method
fn interleave(a: u32, b: u32) -> u64 {
    let mut a = a as u64;
    let mut b = b as u64;
    const MAGIC: [u64; 5] = [
        0x5555555555555555u64,
        0x3333333333333333u64,
        0x0f0f0f0f0f0f0f0fu64,
        0x00ff00ff00ff00ff,
        0x0000ffff0000ffff,
    ];

    for i in (0..5).rev() {
        a = (a | (a << (1 << i))) & MAGIC[i];
        b = (b | (b << (1 << i))) & MAGIC[i];
    }
    a | b << 1
}

/// 32 less significants bits of snd|fst >> k
fn shift(fst: u32, snd: u32, k: u32) -> u32 {
    let x = (snd as u64) << 32 | (fst as u64);
    (x >> k) as u32
}

struct Shifterator<'a> {
    slice: Slice<'a, u32>,
    amount: u32,
}

impl<'a> Iterator for Shifterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() < 2 {
            None
        } else {
            let fst = self.slice[0];
            let snd = self.slice[1];
            Some(shift(fst, snd, self.amount))
        }
    }
}

fn shifterator<'a>(slice: Slice<'a, u32>, amount: usize) -> Shifterator<'a> {
    let completes = amount / 32;
    let partial = amount as u32 % 32;

    Shifterator {
        slice: slice.skip(completes),
        amount: partial,
    }
}

#[inline(always)]
fn xor(a:u32, b:u32) -> u32 {
    a ^ b
}

fn ranger<'a>(input: Slice<'a, u32>, output: Slice<'a, u32>, p: usize, add_one: bool) {
    struct XorMap<I,J>(I,J);
    impl<I: Iterator<Item = u32>,J:Iterator<Item = u32>> Iterator for XorMap<I,J> {
        type Item = u32;
        #[allow(clippy::manual_map)]
        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            match (self.0.next(),self.1.next()) {
                (Some(a),Some(b)) => Some(xor(a,b)),
                _ => None,
            }
        }
    }

    let vanilla = input.iter_val();
    #[inline(always)]
    fn assign<'a>(
        mut output: Slice<'a, u32>,
        mut first: impl Iterator<Item = u32>,
        mut second: impl Iterator<Item = u32>,
    ) {
        let mut output_point = 0;
        while let (Some(fst), Some(snd)) = (first.next(), second.next()) {
            let inter = interleave(fst, snd);
            output[output_point] = inter as u32;
            output[output_point + 1] = (inter >> 32) as u32;
            output_point += 2;
        }
    }

    match (p.is_multiple_of(2), add_one) {
        (true, true) => {
            let mixed = XorMap(input.iter_val(),shifterator(input, p / 2));
            assign(output, mixed, vanilla);
        }
        (true, false) => {
            let mixed = XorMap(shifterator(input, 1),shifterator(input, p / 2 + 1));
            assign(output, vanilla, mixed);
        }
        (false, true) => {
            let mixed = XorMap(input.iter_val(),shifterator(input, p.div_ceil(2)));
            assign(output, vanilla, mixed);
        }
        (false, false) => {
            let mixed = XorMap(vanilla,shifterator(input, p.div_ceil(2)));
            let shifted_vanilla = shifterator(input, 1);
            assign(output, mixed, shifted_vanilla);
        }
    }
}

fn extend(mut input: Slice<u32>, valid: usize, p: usize) {
    if likely(p > 32) {
        let complete = p / 32;
        let partial = p % 32;
        for i in valid..input.len() {
            let fst = input[i - complete - 1];
            let snd = input[i - complete];
            let x = (snd as u64) << 32 | (fst as u64);

            let fst = (x >> (32 - partial)) as u32;
            let snd = (x >> (32 - (partial + 1))) as u32; //OK for values in 0..32 inclusive, partial in 0..31 inclusive so ok

            input[i] = fst ^ snd;
        }
    } else {
        //1 in p+1 pos
        let inserter = 1u64 << p;
        let mut x = (input[valid - 2] as u64) | ((input[valid - 1] as u64) << 32);
        // we keep the p+1 last bits
        x >>= 64 - (p + 1);

        let mut next = || {
            let bit = (x & 1 != 0) ^ (x & 2 != 0);
            x >>= 1;
            if bit {
                x |= inserter;
            }
            bit
        };

        #[allow(clippy::needless_range_loop)]
        for i in valid..input.len() {
            let mut res = 0;
            let mut inserter = (1_u32) << 31;
            for _ in 0..32 {
                if next() {
                    res |= inserter;
                }
                inserter >>= 1;
            }
            input[i] = res;
        }
    }
}

fn step<'a>(input: Slice<'a, u32>, output: Slice<'a, u32>, p: usize, valid: usize, add_one: bool) {
    ranger(input, output, p, add_one);
    extend(output, valid, p);
}

#[derive(Debug)]
pub struct Parametters {
    pub(crate) p: usize,
    pub(crate) valid: usize,
}
fn value_in_init(p: usize, mut n: usize) -> bool {
    if n < 2 {
        return true;
    }
    n -= 2;
    if n < p {
        return false;
    } else if n == p {
        return true;
    }
    n -= p + 1;
    if n < (p - 1) {
        return false;
    } else if n <= p {
        return true;
    }
    false
}

fn init(mut input: Slice<u32>, sign: i32, p: usize) {
    let valid = p.div_ceil(32) + 2;

    if likely(p > 32) {
        // we need less than 3*p values, and we know them!
        let mut counter = 0;
        counter += (1 - sign) as usize;
        #[allow(clippy::needless_range_loop)]
        for i in 0..valid {
            let mut res = 0;
            let mut inserter = (1_u32) << 31;
            for _ in 0..32 {
                if value_in_init(p, counter) {
                    res |= inserter;
                }
                inserter >>= 1;
                counter += 1;
            }
            input[i] = res;
        }
    } else {
        let mut x = 1u64;
        let inserter = 1 << p;

        let mut next = || {
            let bit = x & 1 != 0;
            x >>= 1;
            if bit && (x & 1 != 0) {
                x |= inserter;
            }
            bit
        };
        match sign {
            1 => {}
            0 => {
                next();
            }
            -1 => {
                next();
                next();
            }
            _ => {}
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..valid {
            let mut res = 0;
            let mut inserter = (1_u32) << 31;
            for _ in 0..32 {
                if next() {
                    res |= inserter;
                }
                inserter >>= 1;
            }
            input[i] = res;
        }
    }
    extend(input, valid, p);
}

// assume that input.len() = scratch.len() = param.ranges_size (else, expect panic / incorrect results)
// fill the output range with the max (rightmost / less significant bit in min index) = F(p,n)
// n reprensented as magnitude bits in DECREASSING significance (you can use .iter_bits method of bit_iterator::BitIterable) + sign in separate var
pub fn calculator<'a>(
    mut scratch1: Slice<'a, u32>,
    mut scratch2: Slice<'a, u32>,
    output: Slice<'a, u32>,
    param: Parametters,
    num_steps: usize,
    n: Slice<'a, u32>,
    n_sign: i32,
) {
    if unlikely(num_steps == 0) {
        init(output, n_sign, param.p);
    }

    init(scratch1, n_sign, param.p);
    //handling negatives

    let mut steps = n.iter_bits();
    for _ in 0..(num_steps - 1) {
        step(
            scratch1,
            scratch2,
            param.p,
            param.valid,
            steps.next().unwrap() != 0,
        );

        swap(&mut scratch1, &mut scratch2);
    }
    step(
        scratch1,
        output,
        param.p,
        param.valid,
        steps.next().unwrap() != 0,
    );
}
