# Makefile for building the tock kernel for the Teensy 4

TARGET=thumbv7em-none-eabi
PLATFORM=teensy4

include ../Makefile.common

%.hex: %.elf
	$(Q)$(OBJCOPY) -O ihex $< $@

kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).hex
	$(Q)teensy_loader_cli --mcu=TEENSY40 -w -v $<

# Unsupported; there's no easily-accessible JTAG interface
.PHONY: flash
flash:
	echo "Use 'make program' to program the Teensy 4"
	exit 1

# For testing with a blinky LED
BLINK=../../../libtock-c/examples/blink/build/cortex-m7/cortex-m7.tbf
.PHONY: program
app.elf: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(Q)arm-none-eabi-objcopy --update-section .apps=$(BLINK) $< $@

app: app.elf
	$(Q)$(OBJCOPY) -O ihex $< $@
	teensy_loader_cli --mcu=TEENSY40 -w -v $@
	$(Q)rm $@ $<
