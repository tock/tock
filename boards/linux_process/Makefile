# Makefile for building the tock kernel as a Linux process 
#

# AMD64
CPU=x86_64

TARGET=$(CPU)-unknown-linux-gnu
PLATFORM=linux_process

RUSTC_FLAGS=

APP=../../../libtock-c/examples/c_hello/build/linux/linux.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

include ../Makefile.common

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(KERNEL_WITH_APP)
