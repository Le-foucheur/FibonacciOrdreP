use std::{borrow::BorrowMut, mem::MaybeUninit};

use gmp_mpfr_sys::gmp::{mpz_init, mpz_set_ui, mpz_t};
use std::ffi::CStr;
use gmp_mpfr_sys::gmp::mpz_set_str;

pub fn utils_mpz_init() -> mpz_t {
    let mut mpz_start = unsafe {
        let mut mpz_start = MaybeUninit::uninit();
        mpz_init(mpz_start.as_mut_ptr());
        mpz_start.assume_init()
    };
    mpz_start
}

pub fn mpz_int_from_string(n: String) -> mpz_t {
    let mut mpz_start = utils_mpz_init();
    let temp = n.as_bytes();
    let n_uchar = CStr::from_bytes_with_nul(temp).unwrap();
    let k = n_uchar.as_ptr();
    unsafe {
        mpz_set_str(mpz_start.borrow_mut(), k, 10);
    }
    mpz_start
}
pub fn mpz_int_from_u64(n: u64) -> mpz_t {
    let mut mpz_start = utils_mpz_init();
    unsafe {
        mpz_set_ui(mpz_start.borrow_mut(), n);
    };
    mpz_start
}