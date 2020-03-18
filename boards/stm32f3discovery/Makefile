# Makefile for building the tock kernel for the STM32F3DISCOVERY platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=stm32f3discovery

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

APP=../../../libtock-c/examples/buttons/build/cortex-m4/cortex-m4.tbf
KERNEL=$(CWD)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(CWD)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

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
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
