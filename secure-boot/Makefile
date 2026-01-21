# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

# Top-level Makefile for secure bootloader
# Usage: make BOARD=<board_name> [target]
# Example: make BOARD=nrf52840dk

# Board directory
ifdef BOARD
BOARD_DIR = boards/$(BOARD)

# Check if board exists
ifeq ($(wildcard $(BOARD_DIR)/Makefile),)
$(error Board '$(BOARD)' not found. Run 'make list-boards' to see available boards)
endif
endif

# Default target - show help if no BOARD specified
.PHONY: all
all:
ifndef BOARD
	@$(MAKE) help
else
	@$(MAKE) build
endif

# Build the bootloader for the specified board
.PHONY: build
build:
ifndef BOARD
	@echo "Error: BOARD not specified"
	@echo "Usage: make BOARD=<board_name> build"
	@echo "Run 'make list-boards' to see available boards"
	@exit 1
endif
	@echo "Building bootloader for $(BOARD)io"
	@$(MAKE) -C $(BOARD_DIR) build

# Clean build artifacts for the specified board
.PHONY: clean
clean:
ifndef BOARD
	@echo "Error: BOARD not specified"
	@echo "Usage: make BOARD=<board_name> clean"
	@echo "Or use 'make clean-all' to clean all boards"
	@exit 1
endif
	@echo "Cleaning bootloader for $(BOARD)io"
	@$(MAKE) -C $(BOARD_DIR) clean

# Flash the bootloader to the board
.PHONY: install
install:
ifndef BOARD
	@echo "Error: BOARD not specified"
	@echo "Usage: make BOARD=<board_name> install"
	@exit 1
endif
	@echo "Flashing bootloader to $(BOARD)io"
	@$(MAKE) -C $(BOARD_DIR) flash

# Show size of the bootloader binary
.PHONY: size
size:
ifndef BOARD
	@echo "Error: BOARD not specified"
	@echo "Usage: make BOARD=<board_name> size"
	@exit 1
endif
	@echo "Bootloader size for $(BOARD):"
	@$(MAKE) -C $(BOARD_DIR) size

# Display available boards
.PHONY: list-boards
list-boards:
	@echo "Available boards:"
	@for board in $(notdir $(wildcard boards/*)); do \
		if [ -f "boards/$$board/Makefile" ]; then \
			echo "  - $$board"; \
		fi \
	done

# Help target
.PHONY: help
help:
	@echo "Tock Secure Bootloader Build System"
	@echo ""
	@echo "Usage:"
	@echo "  1. From top-level: make BOARD=<board_name> [target]"
	@echo "  2. From board dir: cd boards/<board_name> && make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all (default)  - Build the bootloader (or show help if no BOARD)"
	@echo "  build          - Build the bootloader"
	@echo "  flash          - Flash bootloader to board"
	@echo "  size           - Show bootloader binary size"
	@echo "  clean          - Clean build artifacts for specified board"
	@echo "  list-boards    - List available boards"
	@echo "  help           - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make BOARD=nrf52840dk build"
	@echo "  make BOARD=nrf52840dk size"
	@echo "  make BOARD=nrf52840dk flash"
	@echo "  make list-boards"
	@echo "  make clean-all"