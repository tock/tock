# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025.

# Makefile for building the tock kernel for the Raspberry Pi Pico 2 board.

include ../Makefile.common

OPENOCD=openocd
OPENOCD_INTERFACE=picoprobe
OPENOCD_OPTIONS=-f openocd-$(OPENOCD_INTERFACE).cfg

BOOTSEL_FOLDER?=/run/media/$(USER)/RP2350

# From layout.ld: flash begins at `bootloader`, applications at `prog`.
FLASH_ADDRESS=0x10000000
APP_ADDRESS=0x10040000

IMAGE=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-image.bin


# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: init
init:
	tockloader local-board set $(PLATFORM) --binary-path $(IMAGE) --arch cortex-m33 --app-address $(APP_ADDRESS) --flash-address $(FLASH_ADDRESS)

# Write the kernel to the flash file with tockloader
.PHONY: write-kernel
write-kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --local-board --address $(FLASH_ADDRESS) $<

.PHONY: flash-openocd
flash-openocd: write-kernel
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "adapter speed 5000" -c "program $(IMAGE) $(FLASH_ADDRESS) verify reset; shutdown;"

.PHONY: flash
flash: write-kernel
	picotool uf2 convert $(IMAGE) -t bin -o $(FLASH_ADDRESS) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Raspberry Pi Pico 2 Flash Drive Folder; echo You can download and flash $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2; fi
