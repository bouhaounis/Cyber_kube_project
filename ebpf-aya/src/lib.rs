#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(target_arch = "bpf")]
include!("programs.rs");

#[cfg(not(target_arch = "bpf"))]
pub fn host_build_placeholder() {}
