# Segger Peripheral Support

This crate does not abstract a specific chip, but rather provides
support libraries for low-level Segger peripherals that are available
on many chips (most Cortex-M chips and possibly others):

- [Segger RTT](https://wiki.segger.com/RTT): Provides a `hil::uart` interface for the Segger RTT interface
