fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // BTF is only understood by the BPF linker. Emitting it for host-side
    // tests makes regular Linux linking fail with "unknown argument '--btf'".
    if target_arch == "bpf" {
        println!("cargo:rustc-link-arg=-Wl,--btf");
    }
}
