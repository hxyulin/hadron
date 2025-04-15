fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let linker_file = format!("targets/hadron_kernel-{}.ld", target);
    println!("cargo:rerun-if-changed={}", linker_file);
    println!("cargo:rustc-link-arg=-T{}", linker_file);
}
