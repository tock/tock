# Makefile for building the tock kernel for the nRF51 development kit

TOCK_ARCH=cortex-m0
TARGET=thumbv6m-none-eabi
PLATFORM=nrf51dk

include ../../Makefile.common

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/$(TARGET)/release/nrf51dk-$(APP).hex: target/$(TARGET)/release/nrf51dk-$(APP)
	@$(OBJCOPY) -Oihex $^ $@

JLINK=JLinkExe
JLINK_OPTIONS+=-device nrf51422 -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

# Upload the kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/nrf51dk.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

.PHONY: program
program: target/$(TARGET)/release/nrf51dk.hex
	$(error Cannot program nRF51DK over USB. Use \'make flash\' and JTAG)
