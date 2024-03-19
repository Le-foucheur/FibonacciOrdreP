use std::{borrow::{Borrow, BorrowMut}, mem::MaybeUninit};

use gmp_mpfr_sys::gmp::{mpz_add, mpz_add_ui, mpz_cmp, mpz_cmp_si, mpz_divexact_ui, mpz_get_si, mpz_get_str, mpz_init, mpz_set_ui, mpz_sub_ui, mpz_t};
use std::ffi::CStr;
use gmp_mpfr_sys::gmp::mpz_set_str;

pub fn utils_mpz_init() -> mpz_t {
    let mpz_start = unsafe {
        let mut mpz_start = MaybeUninit::uninit();
        mpz_init(mpz_start.as_mut_ptr());
        mpz_start.assume_init()
    };
    mpz_start
}

pub fn utils_mpz_to_i64(mpz_start: &mut mpz_t) -> i64 {
    unsafe {
        mpz_get_si(mpz_start.borrow())
    }
}

pub fn utils_mpz_set_string(mut n: String, mpz_start: &mut mpz_t) {
    // Add the null terminator
    n.push('\0');
    let temp = n.as_bytes();
    let n_uchar = CStr::from_bytes_with_nul(temp).unwrap();
    let k = n_uchar.as_ptr();
    unsafe {
        mpz_set_str(mpz_start.borrow_mut(), k, 10);
    }
}

pub fn utils_mpz_to_string(mpz_start: &mut mpz_t) -> String {
    let mut buffer: Vec<u8> = vec![0; 1000];
    unsafe {
        let cstr = CStr::from_ptr(buffer.as_mut_ptr() as *mut i8);
        let ptr = cstr.as_ptr() as *mut i8;
        mpz_get_str(ptr, 10, mpz_start.borrow());
        let s = CStr::from_ptr(ptr).to_str().unwrap();
        s.to_string()
    }
}

pub fn utils_mpz_from_u64(n: u64) -> mpz_t {
    let mut mpz_start = utils_mpz_init();
    unsafe {
        mpz_set_ui(mpz_start.borrow_mut(), n);
    };
    mpz_start
}
pub fn utils_mpz_add_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_add_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n);
    };
}
pub fn utils_mpz_sub_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_sub_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n);
    };
}
pub fn utils_mpz_compare_i64(mpz_start: &mut mpz_t, n: i64) -> i32 {
    unsafe {
        mpz_cmp_si(mpz_start.borrow(), n)
    }    
}
pub fn utils_mpz_divexact_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_divexact_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n as u64);
    }    
}
pub fn utils_mpz_compare_mpz(mpz_start: &mut mpz_t, mpz_end: &mut mpz_t) -> i32 {
    unsafe {
        mpz_cmp(mpz_start.borrow(), mpz_end.borrow())
    }    
}
pub fn utils_mpz_add_mpz(mpz_start: &mut mpz_t, mpz_end: &mut mpz_t) {
    unsafe {
        mpz_add(mpz_start.borrow_mut(), mpz_start.borrow(), mpz_end.borrow());
    };
}