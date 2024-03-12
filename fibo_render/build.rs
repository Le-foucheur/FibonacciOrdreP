// Example custom build script.
fn main() {
    println!("cargo:rustc-link-search=native=/home/julien/code/fibo_render");
    println!("cargo:rustc-link-lib=gmp");
}