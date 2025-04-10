use serde_derive::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootProtocol {
    Limine,
    Multiboot2,
    Linux,
}

impl FromStr for BootProtocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Limine" => Ok(BootProtocol::Limine),
            "Multiboot2" => Ok(BootProtocol::Multiboot2),
            "Linux" => Ok(BootProtocol::Linux),
            _ => Err(format!("Invalid boot protocol: {}", s)),
        }
    }
}

impl std::fmt::Display for BootProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootProtocol::Limine => write!(f, "Limine"),
            BootProtocol::Multiboot2 => write!(f, "Multiboot2"),
            BootProtocol::Linux => write!(f, "Linux"),
        }
    }
}

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
    pub boot_protocol: BootProtocol,
    pub target: Target,
    pub debug: bool,
    pub smp: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            boot_protocol: BootProtocol::Limine,
            target: Target::X86_64,
            debug: false,
            smp: false,
        }
    }
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Self {
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}

pub fn generate_defconfig(path: &PathBuf) {
    println!("Writing defconfig to {}", path.display());
    write_config(path, &Config::default());
}

pub fn write_config(path: &PathBuf, config: &Config) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &config).unwrap();
}
