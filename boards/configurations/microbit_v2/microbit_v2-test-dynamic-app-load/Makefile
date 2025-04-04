# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the BBC microbit v2 board.

include ../../../Makefile.common

PROBE_RS=probe-rs
CARGO_FLASH=cargo flash
OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

TOCKLOADER=tockloader

# Default target for installing the kernel.
.PHONY: install
install: flash


.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(CARGO_FLASH) --chip nRF52833_xxAA --verify --path $<

.PHONY: flash-probe-rs-debug
flash-probe-rs-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(PROBE_RS) download --chip nRF52833_xxAA --verify $<	

.PHONY: flash-openocd-debug
flash-openocd-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"


.PHONY: flash
flash:	$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(CARGO_FLASH) --chip nRF52833_xxAA --verify --path $<

.PHONY: flash-probe-rs
flash-probe-rs: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(PROBE_RS) download --chip nRF52833_xxAA --verify $<	

.PHONY: flash-openocd
flash-openocd: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"


.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) flash $< --address 0x8000 --page-size 512


.PHONY: flash-app
flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(error Use tockloader to load applications)


.PHONY: flash-bootloader
flash-bootloader:
	curl -L --output /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin https://github.com/tock/tock-bootloader/releases/download/microbit_v2-vv1.1.1/tock-bootloader.microbit_v2.vv1.1.1.bin
	$(CARGO_FLASH) --chip  nRF52833_xxAA --binary-format Bin --verify  /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin
	rm /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin

.PHONY: flash-probe-rs-bootloader
flash-probe-rs-bootloader:
	curl -L --output /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin https://github.com/tock/tock-bootloader/releases/download/microbit_v2-vv1.1.1/tock-bootloader.microbit_v2.vv1.1.1.bin
	$(PROBE_RS) download --chip  nRF52833_xxAA --binary-format Bin --verify  /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin
	rm /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin

.PHONY: flash-openocd-bootloader
flash-openocd-bootloader:
	curl -L --output /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin https://github.com/tock/tock-bootloader/releases/download/microbit_v2-vv1.1.1/tock-bootloader.microbit_v2.vv1.1.1.bin
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin; verify_image /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin; reset halt; shutdown;"
	rm /tmp/tock-bootloader.microbit_v2.vv1.1.1.bin
