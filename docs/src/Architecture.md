# Architecture

This document describes the architecture of the kernel.

The core design philosophy of the kernel is to be modular.
This means that each component (subsystem) is designed to be as independent as possible.
For example, the drivers are designed to be independent of the kernel (with some exceptions, such as finding conflicting devices, e.g. the generic fbdev).

## Categories

This is just an arbitrary categorization of the kernel, which sepreated the kernel into different parts (crates), to allow more modular development.

### Kernel Base

The kernel base is the core of the kernel.
This contains the core functionality of the kernel, such as memory management, interrupt handling, and common data structures.

### Drivers (drivers)

The drivers are the core of the kernel.
Each driver is an independent feature in the driver crate, and the crate can be build either as a static relocatable elf shared library (.so), or a rust library (.rlib).

When built as a rust library, the drivers are considered 'built-in' drivers, and are loaded by the kernel at boot time.
Built-in drivers should include a Block device, Filesystem driver, and some sort of graphics device. This allows for the kernel to load the filesystem, which can be used to load the driver modules, and in the case of an error, the framebuffer device can report errors to the user.

However, even if a driver is builtin, it can still be configured to not be loaded by the kernel using command line arguments. (Note: this is not yet implemented)

## Devices

Devices are the main abstraction of the kernel.
A device is a generic interface to a hardware device.
There is currently 1 main device bus type, which is the PCI bus.
But every device contains the generic device struct, which contains general information about the device. Devices manage their own resources, which include memory allocations and memory maps. This results in a bit of unsafe rust code, as the device contains a opache pointer, which drivers should use to store all their state. This allows clean cleanup of the driver, even if the driver crashes without calling its cleanup function.

Drivers for devices are a special part of the kernel, as there are some special requirements for drivers.
- Drivers must be built as a shared library (.so) or rust library (.rlib). (Either one or multiple can be built into a single library)
- Drivers only have access to a limited set of resources, and not the full kernel base. See the [drivers](Drivers.md) document for more information.
- Drivers must be loaded by the kernel at boot time.
- Drivers should not store state in its module (.data / .rodata sections), and should instead store it in a structure which is allocated using the device.

## Memory Management

The kernel uses a 4-level page table, with linear address space. This means that segmentation is not used, and the kernel is able to use all of the available memory.
The higher half of the address space (canonical address >= 0xFFFF_8000_0000_0000) is used for the kernel, and  the lower half can be used by user-space applications.
This design choice allows easy identification of kernel memory vs. user-space memory, and allows for the kernel to use all of the available memory.

NOTE: The kernel is only implemented for x86_64 at the moment, so all the memory management code is x86_64 specific.

The different memory regions also attempt to achieve the same effect, each having a distinct prefix which allows for easy identification of the region (which can be useful for debugging).
Currently, the kernel uses the following memory regions:
 - Kernel Heap
 - Kernel Stack
 - Kernel Text (and data)
 - Page Tables (Recursive)
 - Memory Map (See below)

### The Memory Map

The kernel uses a bitmap to track free pages, with each region of usable memory containing its own bitmap (each bit represents a page).
This allows the kernel to store many non-contiguous memory regions, while preserving the speed of the bitmap operations. However, due to the nature of the bitmap, the kernel uses a minimum of 1/(8 * PAGE_SIZE) of the memory, as each free page requires at least one byte to store. This is 0.003%, which should not be significant for most usecases.

## Interrupt Handling

The kernel uses a kernel IDT, which is a table of interrupt handlers.
TODO: Add more information about the interrupt handling system.

## Booting

Currently, the kernel uses the limine boot protocol, which is a 64-bit protocol.
This makes writing the kernel easier, as the kernel can be written in a single 64-bit binary, instead of requiring to link a 32-bit binary with a 64-bit binary.

On boot, the bootloader loads the kernel into memory, and jumps to the kernel entry point.
The entry point will parse the `base revision` field of the limine boot protocol, to ensure that the kernel was actually booted by limine,
and the kernel limine-api version is supported. After this, the kernel will parse the memory map, using part of the memory map to store the memory map itself, by calculating the size of the memory map (and some additional space for deallocation), and allocating free pages as needed. The framebuffer is parsed and a special logging mechanism is used for the early stages of the kernel (TODO: make this optional, can be configured out, as it technically has a small cost (one comparison check and jump, which could slow down the cpu if branch prediction fails)), known as the fallback logger. The kernel also initializes the gdt and idt after intializing basic output capability, and allocates pages for the new page table (due to how the limine protocol works, we must allocate our own page tables), and then we jump to the stage 2 entry point (after switching page tables and stack), called `limine_stage_2`, which will initialize the heap, initialize the logging system (using proper logging outputs), call ctor constructors, transition from the initial memory maps to a bitmap based frame allocator memory map, it also frees the bootloader reclaimable memory, and jumps to the `kernel_main` function, passing a struct with the rsdp pointer as an argument.

In the `kernel_main` function, the kernel will initialize the ACPI tables, parsing the HPET, and APICS, and PCIe devices (using the MCFG table), and then initialize the drivers for the kernel.

The rest of the kernel is not yet implemented
