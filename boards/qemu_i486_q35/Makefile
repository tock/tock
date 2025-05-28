# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

# Makefile for building the Tock kernel for the qemu-system-i386 `q35`
# platform / machine type.

# Skip auto-installing targets with rustup, since we are using a custom target
NO_RUSTUP := 1

include ../Makefile.common

QEMU_CMD             := qemu-system-i386
WORKING_QEMU_VERSION := 7.2.0

# Peripherals attached by default:
# - 16550 UART (attached to stdio by default)
QEMU_BASE_CMDLINE := \
  $(QEMU_CMD) \
    -cpu 486 \
    -machine q35 \
    -net none \
    -nographic \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04

# Run the kernel inside a qemu-system-i386 "q35" machine type simulation
#
# In order to boot with QEMU's -kernel flag, the Multiboot V1 header (at the beginning of .text)
# must appear within the first 8k of the ELF file. Unfortunately, the ELF emitted by Rust/LLVM does
# not always place .text first.
#
# To address this issue, we use GNU objcopy with no arguments to regenerate the ELF file. This does
# not change the binary properties in any meaningful way, but it does reorder the physical offset of
# sections within the ELF file such that .text comes first and the Multiboot header is discoverable.
#
# If tock fails to boot with the following error: "qemu_i486_q35: cannot execute binary file",
# replace objcopy with the full path to GNU objcopy.
.PHONY: run
run: $(TARGET_PATH)/release/$(PLATFORM).elf
	@echo
	@echo Reordering ELF file sections
	@$(OBJCOPY)  $<
	@echo
	@echo -e "Running $$(qemu-system-i386 --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSION)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) \
	  -kernel $<

# todo: add run-app target
