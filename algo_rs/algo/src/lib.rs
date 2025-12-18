// #![no_std]
#![feature(array_windows, likely_unlikely, iter_array_chunks)]

use core::{
    hint::{likely, unlikely},
    iter::{repeat_n, repeat_with},
    mem::swap,
};
pub mod bit_iterator;

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
fn shift(fst: u32, snd: u32, k: u8) -> u32 {
    let x = (snd as u64) << 32 | (fst as u64);
    (x >> k) as u32
}

fn shifterator(slice: &[u32], amount: usize) -> impl Iterator<Item = u32> {
    let completes = amount / 32;
    let partial = amount as u8 % 32;

    slice[completes..]
        .array_windows::<2>()
        .map(move |&[fst, snd]| shift(fst, snd, partial))
}

fn xor(t: (u32, u32)) -> u32 {
    t.0 ^ t.1
}

fn ranger(input: &[u32], output: &mut [u32], p: usize, add_one: bool) {
    let vanilla = input.iter().copied();
    let out_iter = output.chunks_exact_mut(2);

    fn assign<'a>(
        output: impl Iterator<Item = &'a mut [u32]>,
        first: impl Iterator<Item = u32>,
        second: impl Iterator<Item = u32>,
    ) {
        let mut iterator = output.zip(first.zip(second));
        while let Some(([o1, o2], (fst, snd))) = iterator.next() {
            let inter = interleave(fst, snd);
            *o1 = inter as u32;
            *o2 = (inter >> 32) as u32
        }
    }

    match (p.is_multiple_of(2), add_one) {
        (true, true) => {
            let mixed = vanilla.clone().zip(shifterator(input, p / 2)).map(xor);
            assign(out_iter, mixed, vanilla);
        }
        (true, false) => {
            let mixed = shifterator(input, 1)
                .zip(shifterator(input, p / 2 + 1))
                .map(xor);
            assign(out_iter, vanilla, mixed);
        }
        (false, true) => {
            let mixed = vanilla
                .clone()
                .zip(shifterator(input, p.div_ceil(2)))
                .map(xor);
            assign(out_iter, vanilla, mixed);
        }
        (false, false) => {
            let mixed = vanilla.zip(shifterator(input, p.div_ceil(2))).map(xor);
            let shifted_vanilla = shifterator(input, 1);
            assign(out_iter, mixed, shifted_vanilla);
        }
    }
}

fn extend(input: &mut [u32], valid: usize, p: usize) {
    if likely(p > 32) {
        let complete = p / 32;
        let partial = (p as u8) % 32;
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
        // println!("{x},{valid}");
        let iter = repeat_with(|| {
            let bit = (x & 1 != 0) ^ (x & 2 != 0);
            x >>= 1;
            if bit {
                x |= inserter;
            }
            bit
        });

        for (chunk, out) in iter.array_chunks::<32>().zip(input[valid..].iter_mut()) {
            let mut res = 0;
            for bit in chunk.iter().rev() {
                res <<= 1;
                if *bit {
                    res |= 1;
                }
            }
            *out = res;
        }
    }
}

fn step(input: &[u32], output: &mut [u32], p: usize, valid: usize, add_one: bool) {
    ranger(input, output, p, add_one);
    extend(output, valid, p);
}

#[derive(Debug)]
pub struct Parametters {
    p: usize,
    pub ranges_size: usize,
    valid: usize,
}

pub fn setup(p: usize) -> Parametters {
    //we need at least that much valid blocs for extend to work without out of bound access
    let valid = p.div_ceil(32) + 1;
    //we discard (p.div_ceil(2).div_ceil(32)+1)+1 blocs in shifterator => to produce "valid" valids bloc, we need that much more
    let ranges_size = valid + p.div_ceil(64) + 2;

    Parametters {
        p,
        ranges_size,
        valid,
    }
}

fn init(input: &mut [u32], sign: i8, valid: usize, p: usize) {
    if likely(p > 32) {
        // we need less than 3*p values, and we know them!
        let mut iter = repeat_n(true, 2)
            .chain(repeat_n(false, p))
            .chain(repeat_n(true, 1))
            .chain(repeat_n(false, p - 1))
            .chain(repeat_n(true, 2))
            .chain(repeat_n(false, p - 2))
            .chain([true, false, true].iter().copied());
        match sign {
            1 => {}
            0 => {
                iter.next();
            }
            -1 => {
                iter.next();
                iter.next();
            }
            _ => panic!(),
        }
        for (chunk, out) in iter.array_chunks::<32>().zip(input.iter_mut()) {
            let mut res = 0;
            for bit in chunk.iter().rev() {
                res <<= 1;
                if *bit {
                    res |= 1;
                }
            }
            *out = res;
        }
    } else {
        let mut x = 0b11u64;
        let inserter = 1 << p;

        let mut iter = repeat_with(|| {
            let bit = x & 1 != 0;
            x >>= 1;
            if bit ^ (x & 1 != 0) {
                x |= inserter;
            }
            bit
        });
        match sign {
            1 => {}
            0 => {
                iter.next();
            }
            -1 => {
                iter.next();
                iter.next();
            }
            _ => panic!(),
        }
        for (chunk, out) in iter.array_chunks::<32>().zip(input.iter_mut().take(valid)) {
            let mut res = 0;
            for bit in chunk.iter().rev() {
                res <<= 1;
                if *bit {
                    res |= 1;
                }
            }
            *out = res;
        }
    }
    extend(input, valid, p);
}

// assume that input.len() = scratch.len() = param.ranges_size (else, expect panic / incorrect results)
// fill the output range with the max (rightmost / less significant bit in min index) = F(p,n)
// n reprensented as magnitude bits in DECREASSING significance (you can use .iter_bits method of bit_iterator::BitIterable) + sign in separate var
pub fn calculator<'a>(
    mut scratch1: &'a mut [u32],
    mut scratch2: &'a mut [u32],
    output: &mut [u32],
    param: Parametters,
    n: impl Iterator<Item = bool>,
    is_n_negative: bool,
) {
    let mut n = n.skip_while(|x| !x).peekable();
    if unlikely(param.p==0) {
        let magic = if
        match n.last() {
            None => true,
            Some(bit) => !bit            
        }

        {0x55555555u32} else {0xAAAAAAAAu32};
        for out in output {
            *out = magic
        }
        return;
    }
    
    if unlikely(n.peek().is_none()) {
        init(output, 0, param.valid, param.p);
    } else {
        n.next();

        let sign = if is_n_negative { -1 } else { 1 };
        if unlikely(n.peek().is_none()) {
            init(output, sign, param.valid, param.p);
            return;
        }

        // Finally, after the special cases the main loop!
        init(scratch1, sign, param.valid, param.p);
        //handling negatives
        let mut n = n.map(|val| val ^ is_n_negative);

        let mut add_one = n.next().unwrap();

        for next_add_one in n {
            step(scratch1, scratch2, param.p, param.valid, add_one);

            swap(&mut scratch1, &mut scratch2);
            add_one = next_add_one;
        }
        step(scratch1, output, param.p, param.valid, add_one);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit_iterator::BitIterable;
    //use std::vec::Vec;
    #[test]
    fn it_works() {
        let result = shift(1 << 31, 1, 31);
        assert_eq!(result, 0b11);
        let packed = 0b110010u32;
        let bits = [0, 1, 0, 0, 1, 1].iter().map(|&x| x == 1);
        for (x, y) in bits.zip(packed.iter_bits()) {
            assert_eq!(x, y)
        }

        assert_eq!(0b10100101, interleave(0b11, 0b1100));

        for (i, j) in shifterator([0b11, 0b111, 0].as_slice(), 1).zip([1 + (1 << 31), 0b11].iter())
        {
            assert_eq!(*j, i)
        }
        /*
        let mut output = repeat_n(0, 50).collect::<Vec<_>>();
        init(output.as_mut_slice(), 1, 3, 33);

        let mut output2 = repeat_n(0, 50).collect::<Vec<_>>();
        init(output2.as_mut_slice(), 1, 2, 33);

        assert_eq!(*output, *output2);

        let mut output = repeat_n(0, 50).collect::<Vec<_>>();
        init(output.as_mut_slice(), 0, 3, 34);

        let mut output2 = repeat_n(0, 50).collect::<Vec<_>>();
        init(output2.as_mut_slice(), 0, 3, 34);
        let mut output3 = repeat_n(0, 50).collect::<Vec<_>>();
        ranger(&output2, &mut output3, 34, false);
        extend(&mut output3, 3, 34);

        assert_eq!(*output, *output3);


        let mut output = repeat_n(0, 50).collect::<Vec<_>>();
        init(output.as_mut_slice(), 1, 15, 31);

        let mut output2 = repeat_n(0, 50).collect::<Vec<_>>();
        init(output2.as_mut_slice(), 1, 2, 31);

        assert_eq!(*output, *output2);*/
    }
}
