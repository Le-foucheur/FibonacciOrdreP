// Example custom build script.
fn main() {
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rerun-if-changed=libfibo_mod2.a");
}