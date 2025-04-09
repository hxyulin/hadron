use menuconfig::config::Config;
use std::path::PathBuf;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let linker_file = format!("{}/kernel-link-{}.ld", MANIFEST_DIR, target);
    println!("cargo:rerun-if-changed={}", linker_file);
    println!("cargo:rustc-link-arg=-T{}", linker_file);

    let config_file = std::env::var("CONFIG_FILE").unwrap_or("../kernel_conf.json".to_string());
    println!("cargo:rerun-if-changed={}", config_file);
    let config = Config::from_file(&PathBuf::from(config_file));

    let bootloader = config.boot_protocol.to_string().to_lowercase();
    println!("cargo:rustc-cfg=kernel_bootloader=\"{}\"", bootloader);

    if config.smp {
        // We enable the configuration for SMP
    }
}
