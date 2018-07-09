# Makefile for building the tock kernel for the launchxl development kit

TOCK_ARCH=cortex-m4
TOCK_BOARD=launchxl
TARGET=thumbv7em-none-eabi
PLATFORM=launchxl
OPENOCD ?= openocd
JLINK_EXE ?= JLinkExe
JLINK_DEVICE ?= cc1312r1f3
JLINK_ARGUMENTS ?= -device $(JLINK_DEVICE) -if jtag -speed 4000 -AutoConnect 1 -jtagconf -1,-1

include ../Makefile.common

flash: target/$(TARGET)/release/launchxl.bin
	$(OPENOCD) -f flash-kernel.openocd

flash-ccfg: target/$(TARGET)/release/launchxlccfg.bin
	$(OPENOCD) -f flash-ccfg.openocd

flash-jlink: target/$(TARGET)/release/launchxl.bin
	$(JLINK_EXE) $(JLINK_ARGUMENTS) -CommanderScript flash-kernel.jlink

flash-ccfg-jlink: target/$(TARGET)/release/launchxlccfg.bin
	$(JLINK_EXE) $(JLINK_ARGUMENTS) -CommanderScript flash-ccfg.jlink
