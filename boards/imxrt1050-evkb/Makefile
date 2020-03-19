# Makefile for building the tock kernel for the NUCLEO-429ZI platform
#
APP=../../../libtock-c/examples/c_hello/build/cortex-m7/cortex-m7.tbf
KERNEL=$(CWD)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(CWD)/target/$(TARGET)/debug/$(PLATFORM)-app.axf
TARGET=thumbv7em-none-eabi
PLATFORM=imxrt1050-evkb

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

# OpenOCD requires fullpath
CWD=$(shell pwd)

# sudo is needed for libusb access to work properly

.PHONY: flash-debug
flash-debug: target/$(TARGET)/debug/$(PLATFORM).elf
# 	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).elf
	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(CWD)/$<; verify_image $(CWD)/$<; reset; shutdown"

.PHONY: program
program: target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
