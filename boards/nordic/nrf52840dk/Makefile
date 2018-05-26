# Makefile for building the tock kernel for the nRF development kit

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=nrf52840dk

include ../../Makefile.common

JLINK=JLinkExe
JLINK_OPTIONS+=-device nrf52 -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

# Upload the kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

# Upload the kernel over serial/bootloader
.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).hex
	$(error Cannot program nRF52DK over USB. Use \`make flash\` and JTAG)
