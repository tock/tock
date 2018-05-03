# Makefile for building the tock kernel for the EK-TM4C1294XL platform

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabihf
PLATFORM=ek-tm4c1294xl

OPENOCD = openocd
OPENOCD_OPTIONS = -f openocd.cfg

include ../Makefile.common

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/$(TARGET)/release/$(PLATFORM)-$(APP): target/$(TARGET)/release/$(PLATFORM) apps/$(APP)/build/$(TOCK_ARCH)/app.bin
	@$(OBJCOPY) --update-section .apps=../../userland/examples/$(APP)/build/$(TOCK_ARCH)/app.bin \
	  --set-section-flags .apps=alloc,code \
	  target/$(TARGET)/release/$(PLATFORM) $@

target/$(TARGET)/release/$(PLATFORM)-$(APP).hex: target/$(TARGET)/release/$(PLATFORM)-$(APP)
	@$(OBJCOPY) -Oihex $^ $@

.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).hex
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; reset; shutdown"

.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).hex
	$(error Cannot program over USB)

