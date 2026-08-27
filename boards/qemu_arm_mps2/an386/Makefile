# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the qemu-system-arm `mps2-an386`
# (Cortex-M4) platform / machine type.

include ../../Makefile.common

QEMU_CMD := qemu-system-arm

# Base address of the "prog" (app) flash region; must match chip_layout.ld.
APP_ADDRESS := 0x00040000

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

# Same as `run`, but load an app (or several apps concatenated together,
# e.g. via `cat app1.tbf app2.tbf > apps.bin` — see README.md for the flash
# alignment requirement when combining more than one) at $(APP_ADDRESS).
#
# Uses the `.bin` kernel image (with the placeholder `.apps` section
# stripped by the standard $(OBJCOPY_FLAGS)) rather than the `.elf`: QEMU
# refuses to load two overlapping ROM blobs, and the kernel `.elf`'s own
# 4-byte `.apps` placeholder otherwise collides with $(APP) at the same
# address.
.PHONY: run-app
run-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1) with\n"\
	  " - kernel $<\n"\
	  " - app(s) $(APP)"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -kernel $< -device loader,file=$(APP),addr=$(APP_ADDRESS)
