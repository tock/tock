# Makefile for building the tock kernel as a Linux process 
#

# AMD64
CPU=x86_64

TARGET=$(CPU)-unknown-linux-gnu
PLATFORM=linux_process_x86_64

RUSTC_FLAGS=

APP=../../../libtock-c/examples/c_hello/build/$(CPU)/$(CPU).tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

include ../Makefile.common

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(KERNEL)

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(KERNEL)

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(KERNEL_WITH_APP)
