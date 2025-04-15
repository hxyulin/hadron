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

## Driver Model

The driver model is inspired by the Linux kernel.
First, we will define the linux driver model, then we will define our own driver model.

### Linux Driver Model

Linux drivers are loaded at boot time, and are responsible for initializing the devices.

We will only talk about the PCI drivers, since they are the most common.
PCI drivers are responsible for initializing the PCI devices.

Drivers are registered in a special table, which is a list of all drivers.
When registering a device, we pass the matching PCI IDs, the probe, and remove functions.
The probe function takes in arguments containing a PCIDev, and a PCIDeviceId (which matched the driver).
