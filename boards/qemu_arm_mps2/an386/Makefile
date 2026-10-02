# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the qemu-system-arm `mps2-an386`
# (Cortex-M4) platform / machine type.

include ../../Makefile.common

QEMU_CMD := qemu-system-arm

# Base address of the "prog" (app) flash region; must match chip_layout.ld.
APP_ADDRESS := 0x00040000

# The apps image `tockloader` packs for us. It holds apps only -- the kernel
# is loaded separately, by QEMU's `-kernel` -- so it is based at the start of
# the "prog" region rather than at the start of flash. Covered by the
# top-level `*.bin` .gitignore rule.
LOCAL_BOARD     := mps2_an386
LOCAL_BOARD_BIN := $(LOCAL_BOARD)_apps.bin
TOCKLOADER_ARCH := cortex-m4

# Peripherals attached by default:
# - CMSDK APB UART0 (attached to stdio)
QEMU_BASE_CMDLINE := \
  $(QEMU_CMD) \
    -machine mps2-an386 \
    -nographic \
    -semihosting

# Run the kernel inside a qemu-system-arm "mps2-an386" machine simulation.
.PHONY: run
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1) with\n"\
	  " - kernel $<"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -kernel $<

.PHONY: qemu
qemu: run

# Register this board's apps image as tockloader's "local board", so
# `tockloader install`/`list`/`uninstall` operate on it with no extra flags.
# tockloader stores a single local board for the whole user account, in its
# own user data directory rather than in this tree, so this replaces any
# board already registered. Re-run it when switching between boards.
.PHONY: init
init:
	tockloader local-board set $(LOCAL_BOARD) \
	  --binary-path $(LOCAL_BOARD_BIN) \
	  --arch $(TOCKLOADER_ARCH) \
	  --flash-address $(APP_ADDRESS) \
	  --app-address $(APP_ADDRESS)

# Same as `run`, but also loads whatever apps `tockloader install` has put in
# the apps image. Run `make init` once first; see README.md.
#
# Uses the `.bin` kernel image (with the placeholder `.apps` section stripped
# by the standard $(OBJCOPY_FLAGS)) rather than the `.elf`: QEMU refuses to
# load two overlapping ROM blobs, and the kernel `.elf`'s own 4-byte `.apps`
# placeholder otherwise collides with the apps image at $(APP_ADDRESS).
.PHONY: run-app
run-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1) with\n"\
	  " - kernel $<\n"\
	  " - apps $(LOCAL_BOARD_BIN)"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -kernel $< \
	  -device loader,file=$(LOCAL_BOARD_BIN),addr=$(APP_ADDRESS)
