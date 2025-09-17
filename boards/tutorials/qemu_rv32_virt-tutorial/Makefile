# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

include ../../Makefile.common

QEMU_CMD              := qemu-system-riscv32
WORKING_QEMU_VERSIONS := 8.2.7, 9.1.3, 9.2.3, 10.0.2
BROKEN_QEMU_VERSIONS  := <= 8.1.5


QEMU_BASE_CMDLINE := \
    $(QEMU_CMD) \
    -machine virt \
    -semihosting \
    -global driver=riscv-cpu,property=smepmp,value=true \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    -device virtio-keyboard-device \
    -device virtio-gpu-device \
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

qemu_rv32_virt_tutorial.bin: install

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
.PHONY: run
run: qemu_rv32_virt_tutorial.bin | install
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -bios '$<'
