# Makefile for building the tock kernel for the NUCLEO-446RE platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=nucleo_f429zi

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

APP=../../../libtock-c/examples/tests/hello_loop/build/cortex-m4/cortex-m4.tbf
# APP=../../../libtock-c/examples/tests/console/build/cortex-m4/cortex-m4.tbf
# APP=../../../libtock-c/examples/tests/console_timeout/build/cortex-m4/cortex-m4.tbf
# APP=../../../libtock-c/examples/buttons/build/cortex-m4/cortex-m4.tbf
# APP=../../../libtock-c/examples/blink/build/cortex-m4/cortex-m4.tbf
KERNEL=$(CWD)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(CWD)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
