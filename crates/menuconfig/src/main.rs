use std::path::PathBuf;

use menuconfig::{config::write_config, term};

#[cfg(not(feature = "menuconfig"))]
compile_error!("The `menuconfig` feature is not enabled when compiling this crate as a binary");

fn main() {
    let config_path = PathBuf::from(std::env::args_os().nth(1).expect("No config path"));
    let config = menuconfig::config::Config::from_file(&config_path);
    let config = term::run(config).unwrap();
    println!("Writing config to {}", config_path.display());
    write_config(&config_path, &config).expect("Failed to write config");
}
