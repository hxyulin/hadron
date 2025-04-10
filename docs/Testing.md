# Testing

Testing is a crucial part of the development process.
But it is also very difficult on bare metal.

Here is how we test different parts of the kernel.

## `volatile`

`volatile` is a crate that provides a safe wrapper around volatile memory. It does not inherently use any hardware features of a specific CPU architecture,
therefore we can test it using the `std` feature and the rust default test harness.

## `limine`

`limine` is a crate that provides a safe wrapper around the [Limine bootloader](https://github.com/limine-bootloader/limine).
It is currently not being tested, but it is planned to be tested using a mock kernel.

## `kernel-base`

`kernel-base` is a crate that provides a safe wrapper around the [x86_64 architecture](https://en.wikipedia.org/wiki/X86-64), as well as
other 64-bit architectures (Aarch64, etc..). Due to the nature of the architecture, it is not possible to test it using the `std` feature and the rust default test harness.
Instead, it is tested using the `no_std` feature, using the `hadron-test` crate, providing a mock kernel and test harness with Serial I/O.
