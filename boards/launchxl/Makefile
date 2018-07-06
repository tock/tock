# Makefile for building the tock kernel for the launchxl development kit

TOCK_ARCH=cortex-m4
TOCK_BOARD=launchxl
TARGET=thumbv7em-none-eabi
PLATFORM=launchxl
OPENOCD ?= openocd

include ../Makefile.common

flash: target/$(TARGET)/release/launchxl.hex
	$(OPENOCD) -f flash-kernel.openocd

