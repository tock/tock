# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the Arduino Nano RP2040 Connect board.

include ../Makefile.common

BOOTSEL_FOLDER?=/media/$(USER)/RPI-RP2

# From layout.ld: flash begins at `rom`, applications at `prog`.
FLASH_ADDRESS=0x10000000
APP_ADDRESS=0x10040000

IMAGE=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-image.bin

# Default target for installing the kernel.
.PHONY: install
install: flash

flash-openocd: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<;  reset; shutdown;"

.PHONY: init
init:
	tockloader local-board set $(PLATFORM) --binary-path $(IMAGE) --arch cortex-m0 --app-address $(APP_ADDRESS) --flash-address $(FLASH_ADDRESS)

# Write the kernel to the flash file with tockloader
.PHONY: write-kernel
write-kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --local-board --address $(FLASH_ADDRESS) $<

.PHONY: flash
flash: write-kernel
	picotool uf2 convert $(IMAGE) -t bin -o $(FLASH_ADDRESS) --family rp2040 $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Nano RP2040 Flash Drive Folder; fi
