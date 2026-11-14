# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for the Tock kernel on the ARM MPS3-AN536 platform.

include ../Makefile.common

#TARGET=thumbv8r-none-eabihf #no, A32!

.PHONY: run

# BAD CMD NEED DBG
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	qemu-system-arm \
        -smp 1 \
        -M mps3-an536 \
        -serial null \
        -serial null \
        -serial mon:stdio \
        -device loader,file="$IMG_FLASH_QEMU",addr=0x08000000,force-raw=on
		-bios $< \
		-nographic

