# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

include ../../Makefile.common

QEMU_CMD              := qemu-system-riscv32
WORKING_QEMU_VERSIONS := 8.2.7, 9.1.3, 9.2.3, 10.0.2
BROKEN_QEMU_VERSIONS  := <= 8.1.5

# QEMU's "virt" machine unconditionally reserves two 32 MiB `pflash` (CFI NOR
# flash) windows at 0x2000_0000 and 0x2200_0000, regardless of whether a
# `-drive if=pflash` is attached. We attach a backing file for the first of
# these (unit 0) to use as separate, writable storage (e.g. for staging
# application images) via `PFlashRegion` in the kernel-protection ePMP; see
# `arch/riscv/src/pmp.rs`.
PFLASH_IMAGE := qemu_rv32_virt_tutorial-apps.bin
PFLASH_SIZE  := 32M

# # Create a backing image for pflash unit 0 if it doesn't already exist,
# # filled with 0xFF to mimic the erased state of real NOR flash.
# $(PFLASH_IMAGE):
# 	dd if=/dev/zero bs=$(PFLASH_SIZE) count=1 2>/dev/null  > $@

QEMU_BASE_CMDLINE := \
    $(QEMU_CMD) \
    -machine virt \
    -semihosting \
    -global driver=riscv-cpu,property=smepmp,value=true \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    -device virtio-keyboard-device \
    -device virtio-gpu-device \
    -drive if=pflash,unit=0,format=raw,file=$(PFLASH_IMAGE) \
    $(QEMU_NETDEV_CMDLINE) \
    -serial stdio


# Set this as the default local board to write apps to with tockloader
.PHONY: init
init:
	tockloader local-board set qemu_rv32_virt --binary-path qemu_rv32_virt_tutorial.bin

# Write the kernel to the flash file with tockloader
.PHONY: install
install: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x80000000 $<
	dd if=qemu_rv32_virt_tutorial.bin of=qemu_rv32_virt_tutorial-kernel.bin bs=1M count=1

qemu_rv32_virt_tutorial-kernel.bin: install

# qemu_rv32_virt_tutorial-apps.bin: qemu_rv32_virt_tutorial.bin
# 	dd if=$< of=$@ bs=1M skip=1
# 	truncate -s 32M $@

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
.PHONY: run
run: qemu_rv32_virt_tutorial-kernel.bin qemu_rv32_virt_tutorial-apps.bin | install
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -bios '$<'
