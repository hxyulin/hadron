use std::path::PathBuf;

#[cfg(not(feature = "menuconfig"))]
compile_error!("The `menuconfig` feature is not enabled when compiling this crate as a binary");

fn main() {
    let config_path = PathBuf::from(std::env::args_os().nth(1).expect("No config path"));
    let config = menuconfig::deserialize(&config_path).expect("Failed to read config");
    let config = menuconfig::term::run(config).unwrap();
    println!("Writing config to {}", config_path.display());
    config.serialize(config_path).expect("Failed to write config");
}
