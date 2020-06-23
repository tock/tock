# Makefile for building the tock kernel for the MSP-EXP432P401 launchpad
#
TARGET=thumbv7em-none-eabi
PLATFORM=msp-exp432p401r

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg
APP_START_ADDR=0x20000
APP?=

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: flash-app
flash-app:
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(APP) $(APP_START_ADDR) bin; verify_image_checksum $(APP) $(APP_START_ADDR) bin; reset; shutdown"
