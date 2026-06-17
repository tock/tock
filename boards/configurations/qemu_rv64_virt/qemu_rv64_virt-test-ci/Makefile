# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

include ../../../Makefile.common

QEMU_CMD := qemu-system-riscv64

QEMU_CMDLINE := \
    $(QEMU_CMD) \
    -machine virt \
    -semihosting \
    -global driver=riscv-cpu,property=smepmp,value=true \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    -device virtio-keyboard-device \
    -device virtio-gpu-device \
    -serial stdio \
    $(QEMU_CMDLINE_EXTRA)
    

# -qmp tcp:localhost:44444,server \
#     -chardev socket,id=serial0,host=localhost,port=44445,server=on \
#     -serial chardev:serial0 \



# Set this as the default local board to write apps to with tockloader
.PHONY: init
init:
	tockloader local-board set qemu_rv64_virt --binary-path qemu_rv64_virt_tutorial.bin

# Write the kernel to the flash file with tockloader
.PHONY: install
install: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x80000000 $<

qemu_rv64_virt_tutorial.bin: install

# Run the kernel inside a qemu-riscv64-system "virt" machine type simulation
.PHONY: run
run: qemu_rv64_virt_tutorial.bin | install
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_CMDLINE) -bios '$<'
