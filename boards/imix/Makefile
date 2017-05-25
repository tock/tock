# Makefile for building the tock kernel for the Imix platform

TARGET=thumbv7em-none-eabi
PLATFORM=imix

include ../Makefile.common

OPENOCD=openocd
OPENOCD_CONF=connect.cfg

# upload kernel with openocd
.PHONY: flash
flash: target/$(TARGET)/release/imix
	#$(OPENOCD) -f $(OPENOCD_CONF) -c "program $< verify reset exit"
	$(OPENOCD) -f $(OPENOCD_CONF) -c "init; reset halt; flash write_image $< 0x0 elf; reset; shutdown"

.PHONY: flash-debug
flash-debug: target/$(TARGET)/debug/imix
	$(OPENOCD) -f $(OPENOCD_CONF) -c "init; reset halt; load_image $<; reset; shutdown"

