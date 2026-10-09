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
	tockloader local-board set esp32_c3_devkitm1 --binary-path esp32_c3_devkitm1.bin --arch rv32imc --app-address 0x42100000 --flash-address 0x42000000

# Write the kernel to the flash file with tockloader
.PHONY: write-kernel
write-kernel: $(TARGET_PATH)/release/$(PLATFORM).bin
	tockloader flash --local-board --address 0x42000000 $<

esp32_c3_devkitm1.bin: write-kernel

flash: esp32_c3_devkitm1.bin
	esptool.py --chip esp32c3 write_flash 0x0 esp32_c3_devkitm1.bin

test:
	$(Q)$(CARGO) test $(NO_RUN) --bin $(PLATFORM) --release
