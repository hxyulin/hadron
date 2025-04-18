use core::convert::AsRef;
use serde_derive::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Target {
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    AArch64,
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64" => Ok(Target::X86_64),
            "aarch64" => Ok(Target::AArch64),
            _ => Err(format!("Invalid target: {}", s)),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::X86_64 => write!(f, "x86_64"),
            Target::AArch64 => write!(f, "aarch64"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target: Target,
    /// Whether to enable serial output
    pub serial: bool,
    /// Whether to enable backtraces
    pub backtrace: bool,
    pub debug: bool,
    pub smp: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            target: Target::X86_64,
            serial: false,
            backtrace: false,
            debug: false,
            smp: false,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        toml::from_str(&contents).unwrap()
    }
}

pub fn generate_defconfig(path: &PathBuf) -> Result<(), std::io::Error> {
    println!("Writing defconfig to {}", path.display());
    write_config(path, &Config::default())
}

pub fn write_config(path: &PathBuf, config: &Config) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    let contents = toml::to_string_pretty(config).expect("Failed to serialize config");
    file.write_all(contents.as_bytes())?;
    Ok(())
}
