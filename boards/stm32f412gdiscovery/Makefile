# Makefile for building the tock kernel for the stm32412gdiscovery platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=stm32f412gdiscovery

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(error See README.md and update this section accordingly)
