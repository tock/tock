# Segger Peripheral Support

This crate does not abstract a specific chip, but rather provides
support libraries for low-level Segger peripherals that are available
on many chips (most Cortex-M chips and possibly others):

- [Segger RTT](https://wiki.segger.com/RTT): Provides a `hil::uart` interface for the Segger RTT interface

These support libraries are included as a chip because the implementations largely
mimic traditional hardware peripheral drivers. For example, the RTT library reads and
writes memory that the chip also accesses via the JTAG hardware to implement the RTT
functionality. This interaction between the Tock driver and hardware introduces safety and
correctness issues that chip implementations generally must handle. As such, we
implement these drivers as a chip crate.
