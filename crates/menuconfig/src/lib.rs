pub use kconfig::{Config, ConfigOption};

#[cfg(feature = "menuconfig")]
pub mod term;

pub fn deserialize<P: AsRef<std::path::Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    Config::deserialize(path)
}

pub fn generate_defconfig<P: AsRef<std::path::Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_root("");
    config.serialize(path)
}
