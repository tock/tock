# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

include ../../../Makefile.common

QEMU_CMD              := qemu-system-riscv32

QEMU_CMDLINE_EXTRA ?= \
    -serial stdio \

# Image file used as PFLASH0. QEMU will write to this file if the chip writes to
# pflash.
PFLASH_IMAGE := qemu_rv32_virt-pflash0.bin

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
    -chardev pty,id=hostserial \
    -device virtio-serial-device \
    -device virtconsole,chardev=hostserial \
    $(QEMU_CMDLINE_EXTRA)


# Set this as the default local board to write apps to with tockloader
.PHONY: init
init:
	tockloader local-board set qemu_rv32_virt --binary-path qemu_rv32_virt.bin

$(PFLASH_IMAGE):
	# Create empty file of all 0xFFs
	head -c 33554432 /dev/zero | LC_ALL=C tr '\0' '\377' > $@

# Write the kernel to the flash file with tockloader
.PHONY: install
install: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x80000000 $<

qemu_rv32_virt.bin: install

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
.PHONY: run
run: qemu_rv32_virt.bin | $(PFLASH_IMAGE)
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -bios '$<'
