use std::{borrow::{Borrow, BorrowMut}, mem::MaybeUninit};

use gmp_mpfr_sys::gmp::{mpz_add_ui, mpz_init, mpz_set_ui, mpz_t};

pub fn mpz_int_from_u64(n: u64) -> mpz_t {
    let mut mpz_start = unsafe {
        let mut mpz_start = MaybeUninit::uninit();
        mpz_init(mpz_start.as_mut_ptr());
        mpz_start.assume_init()
    };
    unsafe {
        mpz_set_ui(mpz_start.borrow_mut(), n);
    };
    mpz_start
}

pub fn mpz_int_set_u64(mpz: &mut mpz_t, n: u64) {
    unsafe {
        mpz_set_ui(mpz.borrow_mut(), n);
    }
}