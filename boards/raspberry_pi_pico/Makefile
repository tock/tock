# Makefile for building the tock kernel for the BBC microbit v2 board.

TOCK_ARCH=cortex-m0p
TARGET=thumbv6m-none-eabi
PLATFORM=raspberry_pi_pico

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

APP=../../../libtock-c/examples/blink/build/cortex-m0/cortex-m0.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<;  reset; shutdown;"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown;"

