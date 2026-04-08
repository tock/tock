# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

# Makefile for nRF52840DK secure bootloader

# Binary name
BINARY_NAME = secure-boot-nrf52840dk

# Cargo build settings
CARGO = cargo
BUILD_MODE = release
TARGET = thumbv7em-none-eabi

# Bootloader address
BOOTLOADER_ADDRESS = 0x00000000

# Directories
SECURE_BOOT_ROOT = ../..
TARGET_DIR = $(SECURE_BOOT_ROOT)/target/$(TARGET)/$(BUILD_MODE)
BUILD_DIR = build

# Output files
ELF_FILE = $(TARGET_DIR)/$(BINARY_NAME)
BIN_FILE = $(BUILD_DIR)/$(BINARY_NAME).bin

# Tools
OBJCOPY = arm-none-eabi-objcopy
TOCKLOADER = tockloader

ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

# Cargo flags
ifeq ($(BUILD_MODE),release)
    CARGO_FLAGS = --release
else
    CARGO_FLAGS =
endif

# Default target
.PHONY: all
all: build

# Build the bootloader
.PHONY: build
build:
	$(CARGO) build $(CARGO_FLAGS)
	@mkdir -p $(BUILD_DIR)
	$(OBJCOPY) -O binary $(ELF_FILE) $(BIN_FILE)
	@echo "  ELF: $(ELF_FILE)"
	@echo "  BIN: $(BIN_FILE)"
	@echo ""
	@ls -lh $(BIN_FILE) | awk '{print "  " $$9 ": " $$5}'


# Default target for installing the bootloader
.PHONY: install
install: all flash

# Upload the bootloader over JTAG using JLink
.PHONY: flash
flash: $(BIN_FILE)
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(BOOTLOADER_ADDRESS) --board nrf52dk --jlink $<

# Upload the bootloader over JTAG using OpenOCD
.PHONY: flash-openocd
flash-openocd: $(BIN_FILE)
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(BOOTLOADER_ADDRESS) --board nrf52dk --openocd $<

# Upload the bootloader using probe-rs
.PHONY: flash-probe-rs
flash-probe-rs: build
	@echo "Flashing bootloader to nRF52840DK using probe-rsio"
	probe-rs run --chip nRF52840_xxAA $(ELF_FILE)

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning nRF52840DK bootloaderio"
	$(CARGO) clean
	rm -rf $(BUILD_DIR)

.PHONY: help
help:
	@echo "nRF52840DK Secure Bootloader"
	@echo ""
	@echo "Targets:"
	@echo "  all (default)    - Build the bootloader"
	@echo "  build            - Build the bootloader"
	@echo "  install          - Flash bootloader"
	@echo "  flash            - Flash to board using tockloader and JLink"
	@echo "  flash-openocd    - Flash to board using tockloader and OpenOCD"
	@echo "  flash-probe-rs   - Flash to board using probe-rs"
	@echo "  clean            - Clean build artifacts"
	@echo "  help             - Show this help"