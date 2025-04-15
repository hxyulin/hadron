use std::{fmt::Display, process::Command, str::FromStr};

#[derive(Debug)]
pub enum Task {
    Build,
    Clean,
    Run,
    Menuconfig,
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
            Task::Test => write!(f, "test"),
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let task = Task::from_str(&args.next().unwrap_or("build".to_string())).unwrap();

    match task {
        Task::Build => build(),
        Task::Clean => clean(),
        Task::Run => run(),
        Task::Menuconfig => menuconfig(),
        Task::Test => test(),
    }
}

fn build_deps() {
    println!("Building drivers");
    let mut command = Command::new("cargo");
    command.arg("build");
    command.args(&["--package", "hadron-drivers"]);
    command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
    command.args(&[
        "-Zbuild-std=core,alloc,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
    ]);
    command.status().unwrap();
}

fn build() {
    build_deps();
    println!("Building Hadron kernel");
    let mut command = Command::new("cargo");
    command.arg("build");
    command.args(&["--package", "hadron-kernel"]);
    command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
    command.args(&[
        "-Zbuild-std=core,alloc,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
    ]);
    command.status().unwrap();
}

fn clean() {
    println!("Cleaning Hadron kernel");
    let mut command = Command::new("cargo");
    command.arg("clean");
    command.status().unwrap();
}

fn run() {
    build_deps();
    println!("Running Hadron kernel");
    let mut command = Command::new("cargo");
    command.arg("run");
    command.args(&["--package", "hadron-kernel"]);
    command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
    command.args(&[
        "-Zbuild-std=core,alloc,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
    ]);
    command.status().unwrap();
}

fn menuconfig() {
    println!("Running menuconfig");
    let mut command = Command::new("cargo");
    command.arg("run");
    command.args(&["--package", "menuconfig"]);
    command.args(&["--", env!("CONFIG_FILE")]);
    command.status().unwrap();
}

fn test() {
    println!("Running tests");
    let mut command = Command::new("cargo");
    command.arg("test");
    command.args(&["--package", "hadron-kernel"]);
    command.args(&["--target", "targets/x86_64-unknown-hadron.json"]);
    command.args(&[
        "-Zbuild-std=core,alloc,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
    ]);
    command.status().unwrap();
}
