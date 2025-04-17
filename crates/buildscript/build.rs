//! This is the build script for the Hadron kernel.

use std::env::current_dir;

fn main() {
    init_logger();
    let root_dir = current_dir()
        .unwrap()
        .join("../../")
        .canonicalize()
        .expect("TODO: handle error");
    log::trace!("Building Hadron kernel in {:?}", root_dir);
    let generated_dir = root_dir.join("target/generated");
    if !generated_dir.exists() {
        std::fs::create_dir(&generated_dir).expect("TODO: handle error");
    }
    let config_file = generated_dir.join("config.toml");
    if !config_file.exists() {
        log::info!("Generating default config");
        menuconfig::config::generate_defconfig(&config_file).expect("BUILD: failed to generate config");
    }
    println!("cargo:rustc-env=CONFIG_FILE={}", config_file.display());
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("{} {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

fn init_logger() {
    // It is a super single logger
    let logger = Box::leak(Box::new(Logger));
    log::set_logger(logger).expect("TODO: handle error");
}
