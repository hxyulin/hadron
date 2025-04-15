# Hadron

Hadron is a POSIX compliant, secure, and open-source operating system written in Rust.
Internally, Hadron takes inspiration from Linux, but is not a direct port / fork.

## Building

Building Hadron is supported on Linux and macOS.
To build Hadron, you must have the [Rust toolchain](https://www.rust-lang.org/tools/install).

The buildscript of the project is a rust project located in the `crates/buildscript` directory.
It is also the only default workspace member, so you can run `cargo run -- build` to build the project.

## License

This project is licensed under the GNU General Public License v3.0.
