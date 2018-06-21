# Makefile for building the tock kernel for the nRF development kit

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=nrf52dk

include ../../Makefile.common

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/$(TARGET)/release/nrf52dk-$(APP).hex: target/$(TARGET)/release/nrf52dk-$(APP)
	@$(OBJCOPY) -Oihex $^ $@

JLINK=JLinkExe
JLINK_OPTIONS+=-device nrf52 -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

# Upload the kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/nrf52dk.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

# Upload the kernel over serial/bootloader
.PHONY: program
program: target/$(TARGET)/release/nrf52dk.hex
	$(error Cannot program nRF52DK over USB. Use \`make flash\` and JTAG)
