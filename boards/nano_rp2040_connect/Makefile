# Makefile for building the tock kernel for the Arduino Nano RP2040 Connect board.

TOCK_ARCH=cortex-m0p
TARGET=thumbv6m-none-eabi
PLATFORM=nano_rp2040_connect

include ../Makefile.common

KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

BOOTSEL_FOLDER?=/media/$(USER)/RP2040

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	elf2uf2 $< $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).uf2
	if [ -d $(BOOTSEL_FOLDER) ];
	then
		cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM)-app.uf2 $(BOOTSEL_FOLDER)"
	else
		@echo Please edit the BOOTSEL_FOLDER variable to point to you Nano RP2040 Flash Drive Folder
	fi

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	elf2uf2 $< $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Nano RP2040 Flash Drive Folder; fi

.PHONY: flash-app
flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application) 
endif 
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	elf2uf2 $(KERNEL_WITH_APP) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2
	@if [ -d $(BOOTSEL_FOLDER) ]; then; cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2 "$(BOOTSEL_FOLDER)"; else echo; echo Please edit the BOOTSEL_FOLDER variable to point to you Nano RP2040 Flash Drive Folder; fi


