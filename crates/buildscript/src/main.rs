use std::{fmt::Display, process::Command, str::FromStr};

#[derive(Debug)]
pub enum Task {
    Build,
    Clean,
    Run,
    Menuconfig,
    Defconfig,
    Test,
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "build" => Ok(Task::Build),
            "clean" => Ok(Task::Clean),
            "run" => Ok(Task::Run),
            "menuconfig" => Ok(Task::Menuconfig),
            "defconfig" => Ok(Task::Defconfig),
            "test" => Ok(Task::Test),
            _ => Err(format!("Invalid task: {}", s)),
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Build => write!(f, "build"),
            Task::Clean => write!(f, "clean"),
            Task::Run => write!(f, "run"),
            Task::Menuconfig => write!(f, "menuconfig"),
            Task::Defconfig => write!(f, "defconfig"),
            Task::Test => write!(f, "test"),
        }
    }
}

const CONFIG_PATH: &str = "target/generated/kconfgen.toml";

fn main() {
    let mut args = std::env::args().skip(1);
    let task = Task::from_str(&args.next().unwrap_or("build".to_string())).unwrap();

    match task {
        Task::Build => build(args.collect()),
        Task::Clean => clean(),
        Task::Run => run(args.collect()),
        Task::Menuconfig => menuconfig(),
        Task::Defconfig => defconfig(),
        Task::Test => test(args.collect()),
    }
}
fn build(args: Vec<String>) {
    println!("building kernel...");
    build_kernel("build", args);
}

fn run(args: Vec<String>) {
    println!("running kernel...");
    build_kernel("run", args);
}

fn build_kernel(arg: &str, args: Vec<String>) {
    /*
    if !std::fs::exists(CONFIG_PATH).unwrap_or(false) {
        eprintln!("Failed to read config file at {}", CONFIG_PATH);
        println!("Generate a config using make defconfig");
        return;
    }
    let config = menuconfig::deserialize(CONFIG_PATH).unwrap();
    */
    let mut command = Command::new("cargo");
    command.arg(arg);
    command.args(&["--package", "hadron-kernel"]);
    /*
    if !config.get::<bool>("debug").unwrap() {
        command.args(&["--release"]);
    }
    let mut features: Vec<&'static str> = Vec::new();
    if config.get::<bool>("serial").unwrap() {
        features.push("printk_serial");
    }

    if !features.is_empty() {
        command.args(&["--features", &features.join(",")]);
    }
    */

    command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
    command.args(&[
        "-Zbuild-std=core,alloc,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
    ]);
    command.arg("--");
    command.args(args);
    command.status().unwrap();
}

fn clean() {
    println!("Cleaning Hadron kernel");
    let mut command = Command::new("cargo");
    command.arg("clean");
    command.status().unwrap();
}

fn menuconfig() {
    println!("Running menuconfig");
    let mut command = Command::new("cargo");
    command.arg("run");
    command.arg("--quiet");
    command.args(&["--package", "menuconfig"]);
    command.args(&["--", CONFIG_PATH]);
    command.status().unwrap();
}

fn defconfig() {
    println!("Generating default config file ({})", CONFIG_PATH);
    let config_dir = std::path::Path::new(CONFIG_PATH).parent().unwrap();
    std::fs::create_dir_all(config_dir).unwrap();
    menuconfig::generate_defconfig(CONFIG_PATH).unwrap();
}

fn test(args: Vec<String>) {
    println!("Running tests");
    let packages = [];
    for package in packages {
        let mut command = Command::new("cargo");
        command.arg("test");
        command.args(&["--package", package]);
        command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
        command.args(&[
            "-Zbuild-std=core,alloc,compiler_builtins",
            "-Zbuild-std-features=compiler-builtins-mem",
        ]);
        command.status().unwrap();
    }

    build_kernel("test", args);
}
