# Tock Repository

Tock has several principal code directories.

- **arch**: stores architecture-specific code. I.e., code that
is Cortex-M0 and Cortex-M4 specific. This includes code for performing
context switches and making system calls (trapping from user code to
kernel code).

- **boards**: contains code for specific Tock platforms, such as
the imix, the Hail, and the nrf52dk. This is typically the structure
that defines all of the capsules the kernel has, the code to configure the
MCU's IO pins into the proper states, initializing the kernel and loading
processes. The principal file in this directory is `main.rs`, and the
principal initialization function is `main` (which executes when the MCU
resets after RAM has been initialized). The board code also defines how system
call device identifiers map to capsules, in the `with_driver` function.

- **capsules**: contains MCU-independent kernel extensions that
can build on top of chip-specific implementations of particular peripherals.
Some capsules provide system calls. For example, the `spi` module in capsules
builds on top of a chip's SPI implementation to provide system calls on
top of it.

- **chips**: contains microcontroller-specific code, such as the
implementations of SPI, I2C, GPIO, UART, and other microcontroller-specific
code. The distinction between chips and boards is the difference between
a microcontroller and a full platform. For example, many microcontrollers
have multiple UARTs. Which UART is the principal way to communicate with
Tock, or which is used to control another chip, is defined by how the chip
is placed on board and which pins are exposed. So a chip provides the UART
implementation, but a board defines which UART is used for what.

- **doc**: contains the documentation for Tock, including
specifications for internal interfaces and tutorials.

- **kernel**: contains microcontroller-independent kernel code,
such as the scheduler, processes, and memory management. This directory
and arch are where all core kernel code reside.

- **libraries**: contains libraries that we use internally and share
externally. Several primitives have been created for Tock that we think could
also be useful to other projects. This is a location where each crate is
located.

- **tools**: contains associated tools to help in compilation and
code maintenance, such as checking code formatting, converting binaries,
and build scripts.

- **vagrant**: contains information on how to get Tock running in a
virtual machine-esque environment.
