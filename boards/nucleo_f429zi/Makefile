# Makefile for building the tock kernel for the NUCLEO-429ZI platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=nucleo_f429zi

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application) 
endif
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
