use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub debug: bool,
    pub smp: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
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
