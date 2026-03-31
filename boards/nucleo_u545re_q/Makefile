# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

TOCK_ROOT_DIRECTORY = ../../
PLATFORM = nucleo_u545re_q

# Path to your compiled app
APP_PATH = /mnt/c/Work/munca/libtock-c/examples/blink/build/cortex-m4/cortex-m4.tbf

include ../Makefile.common

# Target path for the final kernel ELF (from Makefile.common)
KERNEL_ELF = $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)

.PHONY: full
full: $(KERNEL_ELF)
	@echo "--- Extracting Kernel Binary ---"
	$(Q)arm-none-eabi-objcopy -O binary $(KERNEL_ELF) kernel.bin
	
	@echo "--- Padding and Merging App ---"
	# Create 256KB padded image with kernel at start
	$(Q)dd if=/dev/zero of=full_merged.bin bs=1K count=256
	$(Q)dd if=kernel.bin of=full_merged.bin conv=notrunc
	# Append app at 256KB mark (0x0C040000)
	$(Q)cat $(APP_PATH) >> full_merged.bin
	
	@echo "--- Generating Unified ELF ---"
	$(Q)arm-none-eabi-objcopy -I binary -O elf32-littlearm -B arm \
	    --change-section-address .data=0x0C000000 \
	    --rename-section .data=.text \
	    full_merged.bin full_merged.elf
	@echo "Done! Final file: boards/nucleo_u545re_q/full_merged.bin (or .elf)"

OPENOCD=openocd

.PHONY: install
install: flash

.PHONY: flash
flash: full
	$(OPENOCD) -c "source [find board/st_nucleo_u5.cfg]; init; reset halt; flash write_image erase full_merged.bin 0x0C000000; reset; shutdown"
