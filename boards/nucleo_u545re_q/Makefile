# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

include ../Makefile.common

OPENOCD=openocd

.PHONY: install
install: flash

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) -c "source [find board/st_nucleo_u5.cfg]; init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"
