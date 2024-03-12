#![allow(dead_code)]

use crate::fibo::{generate_fibo, FiboSequence};

/// Find the length before a repetition of the pattern
/// A repetition occur when there is p odd term
pub fn find_pattern_length(p: u64) -> u64 {
    let mut sequence = FiboSequence::new(p);
    let mut n = p;
    loop {
        // Get the next p term to check if there are odds
        let next = sequence.generate(n + p, n);
        let mut flag = true;
        for i in next {
            if !i {
                flag = false;
                break;
            }
        }
        if flag {
            break;
        } else {
            n += 1;
        }
    }
    return n;
}

pub fn get_patterns_length(p_max: u64) -> Vec<u64> {
    let mut result = vec![];
    for p in 2..p_max {
        result.push(find_pattern_length(p));
    }
    return result;
}

pub fn show_patterns_length(p_max: u64) {
    println!("\nComputation of patterns sizes...");
    let mut p = 2;
    for v in get_patterns_length(p_max) {
        println!("p: {} -> pattern size : {}", p, v);
        p += 1;
    }
    println!();
}

/// Return true if C(n, k) is odd, false otherwise
pub fn check_pascal_oddeven(n: u64, k: u64) -> bool {
    let nbin = format!("{:b}", n);
    let kbin = format!("{:b}", k);
    for i in 0..nbin.len() {
        if kbin.len() <= i {
            return true;
        }
        if kbin.chars().nth(i) != nbin.chars().nth(i) {
            return false;
        }
    }
    return true;
}

pub fn check_conjecture(p: u64) {
    let ap = find_pattern_length(p);
    let sequence = generate_fibo(p, 100);
    for i in 0..10 {
        let mut sum = sequence[i] as u128 + 2 * sequence[i + (3 * p - 1) as usize] as u128;
        for k in 1..p {
            sum += 2 * sequence[i + (k * (p - 1)) as usize] as u128;
        }
        if sequence[i + ap as usize] != sum {
            println!(
                "Error at index {} : {} != {}",
                i + ap as usize,
                sequence[i + ap as usize],
                sum
            );
        }
    }
}
