use crate::{
    fibo::generate_fibo,
    fibo_test::{check_pascal_oddeven, show_patterns_length},
    window_manager::WindowManager,
};
mod fibo;
mod fibo_fast;
mod fibo_test;
mod window_manager;

#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn fibo2_init_thread_pool(size: isize) -> u32;
}

fn main() {
    // show_patterns_length(10);
    // println!("Fibo : {:#?}", generate_fibo(1, 10));
    // println!("Fibo : {:#?}", generate_fibo(2, 10));
    // println!("Fibo : {:#?}", generate_fibo(3, 10));

    // for n in 0..10 {
    //     for k in 0..n {
    //         // Compute C(n, k)
    //         let mut result = 1;
    //         for i in 0..k {
    //             result *= n - i;
    //             result /= i + 1;
    //         }

    //         println!("Pascal odd-even: n={}, k={}, result={}, result2={},  prevision={}", n, k, result, result % 2 == 1, check_pascal_oddeven(n, k));
    //     }
    // }

    unsafe { fibo2_init_thread_pool(0) };

    // Create the window
    let mut rw = WindowManager::new();

    rw.run();
}
