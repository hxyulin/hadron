# Changelog

All notable changes to this project will be documented in this file, with their commit hash, date, and version number (if applicable).

## ce787569f3e99e799af38db532aecbe9a93fb2d7 (2025-04-13)
 - Added support for PCIE scanning.

## fe0d540beba36beb0627157f3d766528449a1d2c (2025-04-13)
 - Kernel is now relocatable, allowing for it to be loaded at any address.
 - Added support for KASLR.

## dc0d301d942155a43c2e099e745f04a2907867f4 (2025-04-12)

 - `BootstrapPageTable` now uses dynamic memory allocation instead of static memory allocation, allowing for more memory to used with the kernel.
 - Tested with real hardware up to 48 GiB of RAM.
