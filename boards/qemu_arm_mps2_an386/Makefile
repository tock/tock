# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the qemu-system-arm `mps2-an386`
# (Cortex-M4) platform / machine type.

include ../Makefile.common

QEMU_CMD := qemu-system-arm

# Peripherals attached by default:
# - CMSDK APB UART0 (attached to stdio)
QEMU_BASE_CMDLINE := \
  $(QEMU_CMD) \
    -machine mps2-an386 \
    -nographic

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
