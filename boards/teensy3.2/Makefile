# Teensy 3.6
CHIP = mk20
MCU=mk20dx256

PLATFORM=teensy

TARGET = thumbv7em-none-eabi
TEENSY_LOADER=teensy_loader_cli

APP ?= examples/blink
APP_NAME = $(shell basename $(APP))

include ../Makefile.common

.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).hex
	$(TEENSY_LOADER) -mmcu=$(MCU) -v -w $<


.PHONY: app
app: target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME).hex
	$(TEENSY_LOADER) -mmcu=$(MCU) -v -w $<


.PHONY: ../../apps/$(APP)/build/$(TOCK_ARCH)/app
../../apps/$(APP)/build/$(TOCK_ARCH)/app:
	@make -C ../../apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)


target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME): target/$(TARGET)/release/$(PLATFORM) ../../apps/$(APP)/build/$(TOCK_ARCH)/app 
	$(Q)$(OBJCOPY) --update-section .apps=../../apps/$(APP)/build/cortex-m4/cortex-m4.bin \
		--set-section-flags .apps=alloc,code \
		target/$(TARGET)/release/$(PLATFORM) $@

target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME).elf: target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME)
	$(Q)cp $^ $@

target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME).hex: target/$(TARGET)/release/$(PLATFORM)-$(APP_NAME).elf
	$(Q)$(OBJCOPY) -Oihex $^ $@
