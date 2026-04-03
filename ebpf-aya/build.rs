fn main() {
    // BTF support - ensure we have BTF enabled
    println!("cargo:rustc-link-arg=-Wl,--btf");
}
