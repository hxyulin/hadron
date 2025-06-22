# Architecture
Hadron is a monolithic kernel, with support for builtin and runtime-loaded modules. It is designed to provide everything required for anything that can be done in the OS, but in a extensible and dynamic way.

## About The Kernel
The kernel handles things like:
 - Reading blocks of block devices (Hard drives, SSDs, disks, CDs, network, etc...)
 - Abstraction of filesystem (allowing basic operations like read, write, execute), with each filesystem having its own support for POSIX, filename sizes, unicode, etc...
 - Parsing executable files (ELF, scripts with shabang)
 - Perform dynamic linking on executable files, while allowing extensible linker settings (e.g. swapping linkers, options)
 - Abstraction of hardware devices (PCIe, USB, Platform (more info later)), and finding drivers for devices.
 - Multitasking (preemptive and/or cooperative)
 - Task seperate stack, memory, environment variables
 - Interprocess Communication (IPC) and synchronization
 - Monitor system resources / process resources
 - Permission levels (e.g. root, user)
 - Task management (terminating, killing, setting priorities)

To handle these tasks, the kernel uses system calls (also called syscalls).
The syscall ABI is not yet stable, and will be documented in another doc.
> TODO: Document Syscall ABI

## Driver Model

There are 2 types of driver modules, either builtin, or loaded at runtime.
For builtin drivers, they are built into the kernel executable, and registered within the .drivers section, with function pointers to the driver's methods. For loaded drivers, they are loaded in the kernel memory, and their method addressess will be loaded instead.

There are 3 main types of devices. A driver could support different drivers, but each device type has its own entrypoint.
The main device type is a PCI device, which is a PCI (or PCIe) device, on the PCIe bus. The kernel will automatically parse the PCIe bus for devices and functions, and then drivers will be matched based on their vendor and product IDs, and/or sub-ids.

Drivers should not store state, but instead allocate memory owned by the device. This state can be retrieved as all driver methods contain the device as a parameter. This is a design choice to allow multiple devices have multiple instances of the same driver (drivers should be designed for one device, and shared driver management can be done in userspace). This also ensures that memory allocated by the driver will not be leaked as it is deallocated by the device when it is powered off, unplugged, or disabled.
