x86 Architecture Support Components
===================================

This crate includes low-level code for x86 32-bit CPU architectures.

This crate implements flat segmentation for memory management. The entire address space of 4Gb is contained in a single unbroken ("flat") memory segment.

Virtual memory is used as an MPU with a 4KB section granularity.
