# Makefile for building the tock kernel for the nRF development kit

TOCK_ARCH=cortex-m3
TOCK_BOARD=sensortag
TARGET=thumbv7m-none-eabi
PLATFORM=sensortag

include ../Makefile.common

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/$(TARGET)/release/sensortag-$(APP): target/$(TARGET)/release/sensortag apps/$(APP)/build/$(TOCK_ARCH)/app.bin
	@$(OBJCOPY) --update-section .apps=../../userland/examples/$(APP)/build/$(TOCK_ARCH)/app.bin \
	  --set-section-flags .apps=alloc,code \
	  target/$(TARGET)/release/sensortag $@

