# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the qemu-system-arm `mps2-an386`
# (Cortex-M4) platform / machine type, configured to test isolated
# nonvolatile storage backed by RAM.

include ../../../Makefile.common

QEMU_CMD := qemu-system-arm

# Base address of the "prog" (app) flash region; must match
# ../../../qemu_arm_mps2/an386/chip_layout.ld.
APP_ADDRESS := 0x00040000

# The apps image `tockloader` packs for us. It holds apps only -- the kernel
# is loaded separately, by QEMU's `-kernel` -- so it is based at the start of
# the "prog" region rather than at the start of flash. Covered by the
# top-level `*.bin` .gitignore rule.
LOCAL_BOARD     := mps2_an386_test_invs
LOCAL_BOARD_BIN := $(LOCAL_BOARD)_apps.bin
TOCKLOADER_ARCH := cortex-m4

# Peripherals attached by default:
# - CMSDK APB UART0 (attached to stdio, multiplexed with the QEMU monitor --
#   C-a c to switch)
# - CMSDK APB UART1 (attached to a host pseudo-terminal, for the second
#   `Console` this configuration exposes at driver number 0x01000001). QEMU
#   prints the pty's path (e.g. `/dev/ttys004` on macOS, `/dev/pts/4` on
#   Linux) to stdout as it starts, as "char device redirected to ... (label
#   serial1)"; connect to it with, e.g., `screen /dev/ttys004 115200`.
#
# Explicit `-serial` arguments are assigned to serial0, serial1, ... in
# order, so UART0 needs an explicit `mon:stdio` here to keep its normal
# `-nographic` behavior once a second `-serial` is added for UART1.
QEMU_BASE_CMDLINE := \
  $(QEMU_CMD) \
    -machine mps2-an386 \
    -nographic \
    -semihosting \
    -serial mon:stdio \
    -serial pty

.PHONY: init
init:
	tockloader local-board set $(LOCAL_BOARD) \
	  --binary-path $(LOCAL_BOARD_BIN) \
	  --arch $(TOCKLOADER_ARCH) \
	  --flash-address $(APP_ADDRESS) \
	  --app-address $(APP_ADDRESS)

.PHONY: run
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1) with\n"\
	  " - kernel $<\n"\
	  " - apps $(LOCAL_BOARD_BIN)"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -kernel $< \
	  -device loader,file=$(LOCAL_BOARD_BIN),addr=$(APP_ADDRESS)
