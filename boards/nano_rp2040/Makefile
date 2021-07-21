# Makefile for building the tock kernel for the Raspberry Pi Pico board.

TOCK_ARCH=cortex-m0p
TARGET=thumbv6m-none-eabi
PLATFORM=nano_rp2040

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf


# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $<; verify_image $<;  reset; shutdown;"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	elf2uf2 $< $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2
	@echo
	@echo "Run cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).uf2 <path to nano rp2040 flash drive>"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application) 
endif 
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	elf2uf2 $(KERNEL_WITH_APP) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2
	@echo
	@echo "Run cp $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.uf2 <path to nano rp2040 flash drive>"

