fn main() {
    // Expose the build target triple as an env var accessible via env!("TARGET")
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
}
