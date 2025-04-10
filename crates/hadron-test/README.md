# Hadron Test

A baremetal test harness for x86_64.
Currently, it only supports the Limine bootloader, and can be tested using [cargo-qemu-runner](https://github.com/Qwinci/cargo-qemu-runner) (A fork maintained by myself at [rust-qemu-runner](https://github.com/hxyulin/rust-qemu-runner)).

## Running Tests

To run tests, you can use the following command:
```bash
RUSTFLAGS="-C link-arg=-Tutil/limine-x86_64-link.ld -C relocation-model=static -C panic=unwind" cargo test -p hadron-test --example basic_test --target x86_64-un
known-none
```
