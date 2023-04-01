# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the Particle Boron

TARGET=thumbv7em-none-eabi
PLATFORM=particle_boron

include ../Makefile.common

TOCKLOADER=tockloader

# Where in the nrf52 flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x00000

# Can be flashed with nrf52dk config
## TODO: Update board to particle_boron when tockloader supports
TOCKLOADER_JTAG_FLAGS = --jlink --board nrf52dk

# Default target for installing the kernel.
.PHONY: install
install: flash

# Upload the kernel over JTAG
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<
