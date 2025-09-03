# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025.

# Makefile for building the tock kernel for the Raspberry Pi Pico 2 board.

include ../Makefile.common

OPENOCD=openocd
OPENOCD_INTERFACE=picoprobe
OPENOCD_OPTIONS=-f openocd-$(OPENOCD_INTERFACE).cfg

BOOTSEL_FOLDER?=/run/media/$(USER)/RP2350

# Default target for installing the kernel.
.PHONY: install
install: program


.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x10000000 --board raspberry_pi_pico_2 $<


.PHONY: init
	tockloader local-board set raspberry_pi_pico --binary-path rpi2.bin --flash-address 0x10000000 --app-address 0x10040000

rpi2.bin: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x10000000 $<

rpi2.uf2: rpi2.bin
	picotool uf2 convert $< $@

.PHONY: program
program: rpi2.uf2
	cp $< "$(BOOTSEL_FOLDER)"
