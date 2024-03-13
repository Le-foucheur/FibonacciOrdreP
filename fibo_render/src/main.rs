use std::env;

use crate::{help::HELP_MESSAGE, window_manager::WindowManager};
mod fibo;
mod fibo_fast;
mod gmp_utils;
mod progressbar;
mod renderer;
mod window_manager;
mod help;
mod constants;

#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn fibo2_init_thread_pool(size: isize) -> u32;
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut n = 0;
    let mut p = 0;
    let mut zoom = 1.0;

    // Remove the first argument (the program name)
    args.remove(0);
    while args.len() != 0 {
        if args[0] == "-n" {
            n = args[1].parse::<u64>().expect(
                "Invalid argument for -n. Please provide a valid number for the sequence length",
            );
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-p" {
            p = args[1]
                .parse::<u64>()
                .expect("Invalid argument for -p. Please provide a valid number for the modulo");
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-zoom" {
            zoom = args[1].parse::<f32>().expect(
                "Invalid argument for -zoom. Please provide a valid number for the zoom factor",
            );
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-izoom" {
            zoom = 1.0 / args[1].parse::<f32>().expect(
                "Invalid argument for -izoom. Please provide a valid number for the zoom factor",
            );
            args.remove(0);
            args.remove(0);
        } else if args[0] == "-h" || args[0] == "--help" {
            println!("{}", HELP_MESSAGE);
            return;
        } else {
            println!("Unknown argument: {}", args[0]);
            return;
        }
    }

    // Initialize the C library
    unsafe { fibo2_init_thread_pool(0) };

    // Create the window
    let mut wm = WindowManager::new(n, p, zoom);
    wm.run();
}
