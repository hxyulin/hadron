# Hadron

Hadron is a POSIX compliant, secure, and open-source operating system written in Rust.

## Building

Building Hadron is supported on Linux and macOS.
To build Hadron, you must have the [Rust toolchain](https://www.rust-lang.org/tools/install), [GNU Make](https://www.gnu.org/software/make/) installed.

The build process requires a configuration to be set up. This can be generated by running `make default-config`.
Once the configuration is generated, you can either modify `generated/config.toml` or run `make menuconfig` to configure the build (using a TUI).

To build Hadron, run `make` (use `-j` to split the build into multiple threads).

## Running

To run Hadron, you must have a QEMU emulator installed.
The emulator must be configured to support the x86_64 architecture.

Once the emulator is installed, you can run Hadron by running `make run` (or `cargo run -p hadron`, although this is not recommended).

## License

This project is licensed under the GNU General Public License v3.0.
