# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

# Shared makefile for building the tock kernel for nRF test boards.

# Path to signing tool
SIGN_KERNEL_DIR = $(TOCK_ROOT_DIRECTORY)tools/build/kernel-signer
SIGN_KERNEL = $(SIGN_KERNEL_DIR)/../../target/release/kernel-signer

# Build signing tool if it doesn't exist
$(SIGN_KERNEL):
	@echo "Building signing tool"
	cd $(SIGN_KERNEL_DIR) && cargo build --release

# Build the ELF, sign it, create the binary
$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM) $(SIGN_KERNEL)
	@echo "Signing kernel ELF"
	$(SIGN_KERNEL) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)
	@echo "Creating binary from signed ELF"
	$(OBJCOPY) --output-target=binary --strip-sections --strip-all --remove-section .apps $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM) $@
	@$(SIZE) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)
	@sha256sum $@
TOCKLOADER=tockloader

# Where in the flash to load the kernel
KERNEL_ADDRESS=0x08000

# Upload programs over uart with tockloader
ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

# Default target for installing the kernel.
.PHONY: install
install: flash

# Upload the kernel over JTAG
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) --board nrf52dk --jlink $<

# Upload the kernel over JTAG using OpenOCD
.PHONY: flash-openocd
flash-openocd: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) --board nrf52dk --openocd $<
	