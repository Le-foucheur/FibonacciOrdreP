use crate::bit_iterator::BitIterable;
use core::{
    hint::{likely, unlikely},
    mem::swap,
};

/// turn abc, def in daebcf  (b will be shifted once more / a rightmost bit will stay there)
/// see https://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN for method
#[inline(always)]
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
#[inline(always)]
fn shift(fst: u32, snd: u32, k: u32) -> u32 {
    let x = (snd as u64) << 32 | (fst as u64);
    (x >> k) as u32
}

#[inline(always)]
fn shifteraccess(buffer: &mut [u32], buffer_start: usize, amount: usize, index: usize) -> u32 {
    let completes = amount / 32;
    let partial = amount as u32 % 32;
    let real_index = buffer_start + completes + index;
    let fst = buffer[real_index];
    let snd = buffer[real_index + 1];
    shift(fst, snd, partial)
}

#[inline(always)]
fn ranger(
    input_buf: &mut [u32],
    input_start: usize,
    valid: usize,
    output_buf: &mut [u32],
    output_start: usize,
    p: usize,
    add_one: bool,
) {
    let max = valid.div_ceil(2);
    match (p.is_multiple_of(2), add_one) {
        (true, true) => {
            for i in 0..max {
                let mut fst = shifteraccess(input_buf, input_start, p / 2, i);
                let snd = input_buf[input_start + i];
                fst ^= snd;
                let out = interleave(fst, snd);
                output_buf[output_start + 2 * i] = out as u32;
                output_buf[output_start + 2 * i + 1] = (out >> 32) as u32;
            }
        }
        (true, false) => {
            for i in 0..max {
                let fst = input_buf[input_start + i];
                let mut snd = shifteraccess(input_buf, input_start, p / 2 + 1, i);
                snd ^= shifteraccess(input_buf, input_start, 1, i);
                let out = interleave(fst, snd);
                output_buf[output_start + 2 * i] = out as u32;
                output_buf[output_start + 2 * i + 1] = (out >> 32) as u32;
            }
        }
        (false, true) => {
            for i in 0..max {
                let fst = input_buf[input_start + i];
                let mut snd = shifteraccess(input_buf, input_start, p.div_ceil(2), i);
                snd ^= fst;
                let out = interleave(fst, snd);
                output_buf[output_start + 2 * i] = out as u32;
                output_buf[output_start + 2 * i + 1] = (out >> 32) as u32;
            }
        }
        (false, false) => {
            for i in 0..max {
                let mut fst = shifteraccess(input_buf, input_start, p.div_ceil(2), i);
                let snd = shifteraccess(input_buf, input_start, 1, i);
                fst ^= input_buf[input_start + i];
                let out = interleave(fst, snd);
                output_buf[output_start + 2 * i] = out as u32;
                output_buf[output_start + 2 * i + 1] = (out >> 32) as u32;
            }
        }
    }
}

#[inline(always)]
fn extend(buffer: &mut [u32], start: usize, size: usize, valid: usize, p: usize) {
    if likely(p > 32) {
        let complete = p / 32;
        let partial = p % 32;
        for i in valid..size {
            let fst = buffer[start + i - complete - 1];
            let snd = buffer[start + i - complete];
            let x = (snd as u64) << 32 | (fst as u64);

            let fst = (x >> (32 - partial)) as u32;
            let snd = (x >> (32 - (partial + 1))) as u32; //OK for values in 0..32 inclusive, partial in 0..31 inclusive so ok

            buffer[start + i] = fst ^ snd;
        }
    } else {
        //1 in p+1 pos
        let inserter = 1u64 << p;
        let mut x = (buffer[start + valid - 2] as u64) | ((buffer[start + valid - 1] as u64) << 32);
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
        for i in valid..size {
            let mut res = 0;
            let mut inserter = (1_u32) << 31;
            for _ in 0..32 {
                if next() {
                    res |= inserter;
                }
                inserter >>= 1;
            }
            buffer[start + i] = res;
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn step(
    input_buf: &mut [u32],
    input_start: usize,
    output_buf: &mut [u32],
    output_start: usize,
    output_size: usize,
    p: usize,
    valid: usize,
    add_one: bool,
) {
    ranger(
        input_buf,
        input_start,
        valid,
        output_buf,
        output_start,
        p,
        add_one,
    );
    extend(output_buf, output_start, output_size, valid, p);
}

#[derive(Debug)]
pub struct Parametters {
    pub(crate) p: usize,
    pub(crate) valid: usize,
}
#[inline(always)]
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

#[inline(always)]
fn init(buffer: &mut [u32], start: usize, size: usize, sign: i32, p: usize) {
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
            buffer[start + i] = res;
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
            buffer[start + i] = res;
        }
    }
    extend(buffer, start, size, valid, p);
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn calculator(
    the_big_buffer: &mut [u32],
    mut scratch1: usize,
    mut scratch2: usize,
    scratch_size: usize,
    output_buffer: &mut [u32],
    output_start: usize,
    output_size: usize,
    param: Parametters,
    num_steps: usize,
    n_sign: i32,
) {
    if unlikely(num_steps == 0) {
        init(output_buffer, output_start, output_size, n_sign, param.p);
    }

    init(the_big_buffer, scratch1, scratch_size, n_sign, param.p);
    //handling negatives

    let mut counter = 6;
    let mut bits = the_big_buffer[5].iter_bits();
    for _ in 0..(num_steps - 1) {
        let add_one = match bits.next() {
            Some(v) => v,
            None => {
                bits = the_big_buffer[counter].iter_bits();
                counter += 1;
                bits.next().unwrap()
            }
        };
        step(
            unsafe{&mut *(the_big_buffer as *mut _)},
            scratch1,
            the_big_buffer,
            scratch2,
            scratch_size,
            param.p,
            param.valid,
            add_one != 0,
        );

        swap(&mut scratch1, &mut scratch2);
    }
    let add_one = match bits.next() {
        Some(v) => v,
        None => {
            bits = the_big_buffer[counter].iter_bits();
            bits.next().unwrap()
        }
    };
    step(
        the_big_buffer,
        scratch1,
        output_buffer,
        output_start,
        output_size,
        param.p,
        param.valid,
        add_one != 0,
    );
}
