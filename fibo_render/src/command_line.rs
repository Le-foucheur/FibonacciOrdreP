use num::{BigInt, Num};

pub const HELP_MESSAGE: &str = "Command line usage:
    -n <n_start_index>: start index of the sequence (default=0)
    -p <p_start_index>: start index of the p order (default=0)
    -zoom <zoom_factor>: zoom factor (default=1.0)
    -izoom <izoom_factor>: invert zoom factor
    -m, --mode <mode>: rendering mode (0 or 1) (default=0)
    -i, --image, --headless: headless mode (no graphical rendering) :
        -w, --width <width>: width of the image
        -h, --height <height>: height of the image
        -o, --output <filename>: output filename
    --help: display this help message
    
Graphical usage:
    Left/Right/Up/Down arrow: move start index for n and p
        +Ctrl: Move to the next/previous power of two (Placed at the center of the screen)
        +Maj: Move by one
    Z/S: zoom/dezoom
    L: Show/Hide patterns lines
    P: print current render in an image file
    M: Switch between rendering modes
    N: Input n start index
    B: Input p start index
    Ctrl+Q: Quit the program
    H: display this help message
";

// Command line arguments helper functions
fn _start_load_argument(args: &mut Vec<String>, msg: &str) -> bool {
    if args.len() < 2 {
        println!("{}", msg);
        return true;
    }
    return false;
}
fn _end_argument(args: &mut Vec<String>) {
    args.remove(0);
    args.remove(0);
}

pub fn load_argument_u64(args: &mut Vec<String>, arg: &mut u64, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    *arg = args[1].replace("_", "").parse::<u64>().expect(msg);
    _end_argument(args);
}
pub fn load_argument_u32(args: &mut Vec<String>, arg: &mut u32, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    *arg = args[1].replace("_", "").parse::<u32>().expect(msg);
    _end_argument(args);
}
pub fn load_argument_u8(args: &mut Vec<String>, arg: &mut u8, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    *arg = args[1].replace("_", "").parse::<u8>().expect(msg);
    _end_argument(args);
}
pub fn load_argument_f32(args: &mut Vec<String>, arg: &mut f32, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    *arg = args[1].parse::<f32>().expect(msg);
    _end_argument(args);
}
pub fn load_argument_bigint(args: &mut Vec<String>, arg: &mut BigInt, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    let mut temp = args[1].parse::<String>().expect(msg);
    temp = temp.replace("_", "");

    *arg = BigInt::from_str_radix(temp.as_str(), 10).unwrap();
    _end_argument(args);
}

pub fn load_argument_string(args: &mut Vec<String>, arg: &mut String, msg: &str) {
    if _start_load_argument(args, msg) {
        return;
    }
    *arg = args[1].parse::<String>().expect(msg);
    _end_argument(args);
}
