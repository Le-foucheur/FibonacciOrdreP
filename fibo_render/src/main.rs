use std::env;

use command_line::{load_argument_f32, load_argument_u32, load_argument_u64, load_argument_u8, load_argument_mpz};
use gmp_utils::utils_mpz_to_u64;

use crate::{command_line::HELP_MESSAGE, window_manager::WindowManager, gmp_utils::utils_mpz_init};
mod fibo;
mod fibo_fast;
mod gmp_utils;
mod progressbar;
mod renderer;
mod window_manager;
mod command_line;
mod constants;

#[link(name = "fibo_mod2", kind = "static")]
extern "C" {
    fn fibo2_init_thread_pool(size: isize) -> u32;
}


fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut n = 0;
    let mut n_mpz = utils_mpz_init();
    let mut p = 0;
    let mut zoom = 1.0;
    let mut mode = 0;
    
    // Headless default parameters
    let mut headless = false;
    let mut height = 1080;
    let mut width = 1920;

    // Remove the first argument (the program name)
    args.remove(0);
    while args.len() != 0 {
        if args[0] == "-n" {
            load_argument_mpz(
                &mut args,
                &mut n_mpz,
                "Invalid argument for -n. Please provide a valid number for the sequence length",
            );
            n = utils_mpz_to_u64(&mut n_mpz);
        } else if args[0] == "-p" {
            load_argument_u64(
                &mut args,
                &mut p,
                "Invalid argument for -p. Please provide a valid number for the starting p value",
            );
        } else if args[0] == "-zoom" {
            load_argument_f32(
                &mut args,
                &mut zoom,
                "Invalid argument for -zoom. Please provide a valid number for the zoom factor",
            );
        } else if args[0] == "-izoom" {
            load_argument_f32(
                &mut args,
                &mut zoom,
                "Invalid argument for -izoom. Please provide a valid number for the zoom factor",
            );
            zoom = 1.0 / zoom;
        } else if args[0] == "-m" || args[0] == "--mode" {
            load_argument_u8(
                &mut args,
                &mut mode,
                "Invalid argument for -m. Please provide a valid number for the mode",
            );
        } else if args[0] == "-i" || args[0] == "--image" || args[0] == "--headless" {
            headless = true;
            args.remove(0);
        } else if args[0] == "-w" || args[0] == "--width" {
            load_argument_u32(
                &mut args,
                &mut width,
                "Invalid argument for -w. Please provide a valid number for the width",
            );
        } else if args[0] == "-h" || args[0] == "--height" {
            load_argument_u32(
                &mut args,
                &mut height,
                "Invalid argument for -h. Please provide a valid number for the height",
            );
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

    // Create the renderer
    let mut renderer = renderer::Renderer::new(zoom, n, n_mpz, p, mode);

    if headless {
        renderer.generate_sequences_texture(width, height, None);
        renderer.save_image();
        return;
    } else {
        // Create the window
        let mut wm = WindowManager::new(renderer);
        wm.run();
    }
}
