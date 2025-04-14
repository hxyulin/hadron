# Changelog

All notable changes to this project will be documented in this file, with their commit hash, date, and version number (if applicable).

## 48009992f2e3d1d6ed130e114a5e78e9a243d268 (2025-04-14)
 - Seperated BootInfo and RuntimeInfo into two structs.
 - The logger now works universally using a fallback logger.

## 179527b2f3f281da42ad6312054bd4a64f090eba (2025-04-13)
 - Removed code for old device and driver models.

## ce787569f3e99e799af38db532aecbe9a93fb2d7 (2025-04-13)
 - Added support for PCIE scanning.

## fe0d540beba36beb0627157f3d766528449a1d2c (2025-04-13)
 - Kernel is now relocatable, allowing for it to be loaded at any address.
 - Added support for KASLR.

## dc0d301d942155a43c2e099e745f04a2907867f4 (2025-04-12)
 - `BootstrapPageTable` now uses dynamic memory allocation instead of static memory allocation, allowing for more memory to used with the kernel.
 - Tested with real hardware up to 48 GiB of RAM.
