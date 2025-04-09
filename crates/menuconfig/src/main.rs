use std::path::PathBuf;

use clap::Parser;
use menuconfig::{
    config::{generate_defconfig, write_config},
    term,
};

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    path: PathBuf,
    #[clap(short = 'g', long)]
    generate_defconfig: bool,
}

#[cfg(not(feature = "menuconfig"))]
compile_error!("The `menuconfig` feature is not enabled when compiling this crate as a binary");

fn main() {
    let args = Args::parse();
    if args.generate_defconfig {
        generate_defconfig(&args.path);
    } else {
        let config = term::run().unwrap();
        println!("Writing config to {}", args.path.display());
        write_config(&args.path, &config);
    }
}
