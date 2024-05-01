#![allow(non_camel_case_types, non_snake_case)]
use std::{
    borrow::{Borrow, BorrowMut},
    mem::MaybeUninit,
    ptr::NonNull,
};

// pub use gmp_mpfr_sys::gmp::mpz_t;
#[cfg(feature = "graphic")]
use std::ffi::c_long;
#[cfg(feature = "graphic")]
use libc::FILE;
use std::ffi::CStr;
use std::ffi::{c_char, c_int, c_ulong};
extern crate libc;
extern crate libc_stdhandle;
// Link to lib funcs
/// This part is copied from the "gmp_mpfr_sys" crate

type GMP_LIMB_T = c_ulong;
pub type limb_t = GMP_LIMB_T;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct mpz_t {
    pub alloc: c_int,
    pub size: c_int,
    pub d: NonNull<limb_t>,
}
pub type mpz_ptr = *mut mpz_t;
pub type mpz_srcptr = *const mpz_t;
extern "C" {
    #[link_name = "__gmpz_init"]
    pub fn mpz_init(x: mpz_ptr);

    #[link_name = "__gmpz_add"]
    pub fn mpz_add(rop: mpz_ptr, op1: mpz_srcptr, op2: mpz_srcptr);
    #[link_name = "__gmpz_get_str"]
    pub fn mpz_get_str(str: *mut c_char, base: c_int, op: mpz_srcptr) -> *mut c_char;
    #[link_name = "__gmpz_set_ui"]
    pub fn mpz_set_ui(rop: mpz_ptr, op: c_ulong);
    #[link_name = "__gmpz_set_str"]
    pub fn mpz_set_str(rop: mpz_ptr, str: *const c_char, base: c_int) -> c_int;
    #[link_name = "__gmpz_sizeinbase"]
    pub fn mpz_sizeinbase(arg1: mpz_srcptr, arg2: c_int) -> usize;
}
#[cfg(feature = "graphic")]
extern "C" {
    #[link_name = "__gmpz_add_ui"]
    pub fn mpz_add_ui(rop: mpz_ptr, op1: mpz_srcptr, op2: c_ulong);
    #[link_name = "__gmpz_cmp"]
    pub fn mpz_cmp(op1: mpz_srcptr, op2: mpz_srcptr) -> c_int;
    #[link_name = "__gmpz_cmp_si"]
    pub fn mpz_cmp_si(op1: mpz_srcptr, op2: c_long) -> c_int;
    #[link_name = "__gmpz_divexact_ui"]
    pub fn mpz_divexact_ui(q: mpz_ptr, n: mpz_srcptr, d: c_ulong);
    #[link_name = "__gmpz_get_si"]
    pub fn mpz_get_si(op: mpz_srcptr) -> c_long;
    #[link_name = "__gmpz_sub_ui"]
    pub fn mpz_sub_ui(rop: mpz_ptr, op1: mpz_srcptr, op2: c_ulong);
    #[link_name = "__gmpz_mul_ui"]
    pub fn mpz_mul_ui(rop: mpz_ptr, op1: mpz_srcptr, op2: c_ulong);
    #[link_name = "__gmpz_inp_str"]
    pub fn mpz_inp_str(rop: mpz_ptr, stream: *mut FILE, base: c_int) -> c_int;
}

// Utils functions

pub fn utils_mpz_init() -> mpz_t {
    let mpz_start = unsafe {
        let mut mpz_start = MaybeUninit::uninit();
        mpz_init(mpz_start.as_mut_ptr());
        mpz_start.assume_init()
    };
    mpz_start
}

#[cfg(feature = "graphic")]
pub fn utils_mpz_to_i64(mpz_start: &mut mpz_t) -> i64 {
    unsafe { mpz_get_si(mpz_start.borrow()) }
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
    let size = unsafe { mpz_sizeinbase(mpz_start.borrow(), 10) };
    let mut buffer: Vec<u8> = vec![0; size + 5];
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
#[cfg(feature = "graphic")]
pub fn utils_mpz_add_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_add_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n);
    };
}
#[cfg(feature = "graphic")]
pub fn utils_mpz_sub_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_sub_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n);
    };
}
#[cfg(feature = "graphic")]
pub fn utils_mpz_compare_i64(mpz_start: &mut mpz_t, n: i64) -> i32 {
    unsafe { mpz_cmp_si(mpz_start.borrow(), n) }
}
#[cfg(feature = "graphic")]
pub fn utils_mpz_divexact_u64(mpz_start: &mut mpz_t, n: u64) {
    unsafe {
        mpz_divexact_ui(mpz_start.borrow_mut(), mpz_start.borrow(), n as u64);
    }
}
#[cfg(feature = "graphic")]
pub fn utils_mpz_compare_mpz(mpz_start: &mut mpz_t, mpz_end: &mut mpz_t) -> i32 {
    unsafe { mpz_cmp(mpz_start.borrow(), mpz_end.borrow()) }
}

#[cfg(feature = "graphic")]
pub fn utils_mpz_set_stdin(mpz_start: &mut mpz_t) {
    unsafe {
        mpz_inp_str(mpz_start.borrow_mut(), libc_stdhandle::stdin(), 10);
    }
}

pub fn utils_mpz_add_mpz(mpz_start: &mut mpz_t, mpz_end: &mut mpz_t) {
    unsafe {
        mpz_add(mpz_start.borrow_mut(), mpz_start.borrow(), mpz_end.borrow());
    };
}
