# Makefile for building the tock kernel for the Pico Explorer Base board.

TOCK_ARCH=cortex-m0p
TARGET=thumbv6m-none-eabi
PLATFORM=raspberry_pi_pico

include ../Makefile.common

OPENOCD=openocd
OPENOCD_INTERFACE=swd
OPENOCD_OPTIONS=-f openocd-$(OPENOCD_INTERFACE).cfg

BOOTSEL_FOLDER?=/media/$(USER)/RPI-RP2

KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf


# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<;  reset; shutdown;"

.PHONY: flash
flash: $(KERNEL)
	elf2uf2 $< $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Raspberry Pi Pico Flash Drive Folder; echo You can download and flash $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2; fi

.PHONY: program
program: $(KERNEL)
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application) 
endif 
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	elf2uf2 $(KERNEL_WITH_APP) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Raspberry Pi Pico Flash Drive Folder; echo You can download and flash $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2; fi

