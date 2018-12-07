# Makefile for building the tock kernel for the NUCLEO-446RE platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=nucleo_f446re

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

# OpenOCD requires fullpath
CWD=$(shell pwd)

# sudo is needed for libusb access to work properly

.PHONY: flash-debug
flash-debug: target/$(TARGET)/debug/$(PLATFORM).elf
	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).elf
	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: program
program: target/$(TARGET)/debug/$(PLATFORM).elf
	$(error See README.md and update this section accordingly)
