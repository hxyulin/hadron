# Features

Features of the kernel, updated live as the kernel is developed.

## General
 - Support for in theory infinite memory (up to x86_64 physical address space, tested up to 48 GiB of RAM).
    - This is accomplished by using dynamic memory allocation instead of static memory allocation, allowing for more memory to be used with the kernel.
    - BootstrapPageTable, BootstrapMemoryMap, MemoryMap all are dynamic, allowing for more memory to be used with the kernel.

## Optimizations
 - Fast frame allocation.
    - Frames are allocated from a bitmapped frame allocator, which is much faster than a traditional linked list / memory region allocator, which risk fragmentation.
    - However, this comes at the cost of a larger memory footprint (1 bit per frame = 1 byte 32768 frames), around 0.003% of the memory available.
