fn main() {
    // Force the macOS Apple Silicon linker to look in the local library folder
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}