# Makefile for building the tock kernel for the NUCLEO-429ZI platform
#
APP=../../../libtock-c/examples/c_hello/build/cortex-m7/cortex-m7.tbf
TARGET=thumbv7em-none-eabi
PLATFORM=imxrt1050-evkb

KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.axf

include ../Makefile.common

# OPENOCD=openocd
# OPENOCD_OPTIONS=-f openocd.cfg

# OpenOCD requires fullpath
# CWD=$(shell pwd)

# sudo is needed for libusb access to work properly

.PHONY: $(TOCK_ROOT_DIRECTORY)flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(error See README.md file. You will need to use MCUExpresso in order to flash on the device.)

.PHONY: $(TOCK_ROOT_DIRECTORY)flash
flash: target/$(TARGET)/release/$(PLATFORM).elf
	$(error See README.md file. You will need to use MCUExpresso in order to flash on the device.)

.PHONY: $(TOCK_ROOT_DIRECTORY)program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(warning Tock OS was compiled. In order to flash it on the board, you will need MCUExpresso. See README.md for details.)