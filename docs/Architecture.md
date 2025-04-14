# Architecture

## Kernel

The kernel is written in Rust, and is designed to be a monolithic kernel.
Drivers can either be loaded as modules or as part of the kernel.

## Memory

The kernel uses a flat memory model, with a 4-level page table.
Higher half addresses are used for the kernel and kernel data.

## Devices

The kernel has a PCI bus driver, which is responsible for handling all PCI devices.
The devices are enumerated into a DeviceTree, which contains the devices in a tree structure.
