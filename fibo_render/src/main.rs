use crate::window_manager::WindowManager;
mod fibo;
mod fibo_fast;
mod gmp_utils;
mod window_manager;

#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn fibo2_init_thread_pool(size: isize) -> u32;
}

fn main() {
    // Initialize the C library
    unsafe { fibo2_init_thread_pool(0) };

    // Create the window
    let mut wm = WindowManager::new();
    wm.run();
}
