# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the CLUE nRF52840 board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=clue_nrf52840

include ../Makefile.common

ifdef PORT
  FLAGS += -p $(PORT)
endif

# Upload the kernel using nrfutil
.PHONY: program program-apps

# Default target for installing the kernel.
.PHONY: install
install: program

program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader $(FLAGS) flash --address 0x36000 $<

flash-bootloader:
ifeq ($(PORT),) 
	$(error Please specify the serial port using the PORT variable)
endif
	curl -L --output /tmp/tock-bootloader.clue_nrf52840.1.1.2.bin https://github.com/tock/tock-bootloader/releases/download/clue_nrf52840-1.1.2/tock-bootloader.clue_nrf52840.1.1.2.bin
	adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00B6 --application bootloader/tock-bootloader.clue_nrf52840.1.1.2.bin /tmp/tock-bootloader.clue_nrf52840.1.1.2.zip
	adafruit-nrfutil --verbose dfu serial -pkg /tmp/tock-bootloader.clue_nrf52840.1.1.2.zip $(FLAGS) -b 115200 --singlebank --touch 1200

flash-kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
ifeq ($(PORT),) 
	$(error Please specify the serial port using the PORT variable)
endif
	adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00B6 --application $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).zip
	adafruit-nrfutil --verbose dfu serial -pkg $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).zip $(FLAGS) -b 115200 --singlebank --touch 1200
