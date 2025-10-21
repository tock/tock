# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the ESP32-C3 platform

RISC_PREFIX = riscv64-linux-gnu

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: init
init:
	tockloader local-board set esp32_c3_devkitm1 --binary-path esp32_c3_devkitm1.bin --arch rv32imc --app-address 0x403B0000 --flash-address 0x40380000

# Write the kernel to the flash file with tockloader
.PHONY: write-kernel
write-kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --local-board --address 0x40380000 $<

esp32_c3_devkitm1.bin: write-kernel

flash: esp32_c3_devkitm1.bin
	# esp32_c3_devkitm1.bin contains the kernel and all apps. Convert that to a
	# very simple ELF file.
	$(OBJCOPY) --input-target=binary --output-target=elf32-littleriscv --change-section-address=.data=0x40380000 --set-start=0x40380000 esp32_c3_devkitm1.bin esp32_c3_devkitm1.bin.elf
	# Convert our simple ELF file with the kernel and apps to an ESP32-C3
	# compatible binary we can flash to the board.
	esptool.py --chip esp32c3 elf2image --output esp32_c3_devkitm1.flash.bin esp32_c3_devkitm1.bin.elf --dont-append-digest
	# Write the ESP32-C3 binary to the board.
	esptool.py --chip esp32c3 write_flash --flash_mode dio --flash_size detect --flash_freq 80m 0x0 esp32_c3_devkitm1.flash.bin

test:
	$(Q)$(CARGO) test $(NO_RUN) --bin $(PLATFORM) --release
