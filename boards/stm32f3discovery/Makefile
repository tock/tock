# Makefile for building the tock kernel for the STM32F3DISCOVERY platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=stm32f3discovery

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

# OpenOCD requires fullpath
CWD=$(shell pwd)

# sudo is needed for libusb access to work properly

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(error See README.md and update this section accordingly)
