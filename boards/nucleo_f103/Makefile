TOCK_ARCH = cortex-m3
TARGET = thumbv7m-none-eabi
PLATFORM = nucleo_f103

OPENOCD = openocd
OPENOCD_OPTIONS = -f openocd.cfg

include ../Makefile.common

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/$(TARGET)/release/nucleo_f103-$(APP): target/$(TARGET)/release/nucleo_f103 apps/$(APP)/build/$(TOCK_ARCH)/app.bin
	@$(OBJCOPY) --update-section .apps=../../userland/examples/$(APP)/build/$(TOCK_ARCH)/app.bin \
	  --set-section-flags .apps=alloc,code \
	  target/$(TARGET)/release/nucleo_f103 $@

target/$(TARGET)/release/nucleo_f103-$(APP).hex: target/$(TARGET)/release/nucleo_f103-$(APP)
	@$(OBJCOPY) -Oihex $^ $@

.PHONY: flash
flash: target/$(TARGET)/release/nucleo_f103.hex
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; reset; shutdown"

.PHONY: program
program: target/$(TARGET)/release/nucleo_f103.hex
	$(error Cannot program over USB)
