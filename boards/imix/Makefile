# Makefile for building the tock kernel for the Imix platform

SIZE?=arm-none-eabi-size
OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

#OPENOCD=openocd
OPENOCD=/opt/gnuarmeclipse/openocd/0.10.0-201610281609-dev/bin/openocd
OPENOCD_CONF=connect.cfg

.PHONY: all
all: target/sam4l/release/imix

.PHONY: doc
doc:
	@cargo doc --release --target=sam4l.json

.PHONY: target/sam4l/release/imix
target/sam4l/release/imix:
	@cargo build --release --target=sam4l.json
	@$(SIZE) $@

.PHONY: target/sam4l/debug/imix
target/sam4l/debug/imix:
	@cargo build --target=sam4l.json
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > target/sam4l/debug/imix.lst
	@$(SIZE) $@

target/sam4l/release/imix.hex: target/sam4l/release/imix
	@$(OBJCOPY) -Oihex $^ $@

target/sam4l/debug/imix.hex: target/sam4l/debug/imix
	@$(OBJCOPY) -Oihex $^ $@

.PHONY: clean
clean::
	@cargo clean

.PHONY: debug
debug: target/sam4l/debug/imix

# upload kernel with openocd
.PHONY: flash
flash: target/sam4l/release/imix
	#$(OPENOCD) -f $(OPENOCD_CONF) -c "program $< verify reset exit"
	$(OPENOCD) -f $(OPENOCD_CONF) -c "init; reset halt; flash write_image $< 0x0 elf; reset; shutdown"

.PHONY: flash-debug
flash-debug: target/sam4l/debug/imix
	$(OPENOCD) -f $(OPENOCD_CONF) -c "init; reset halt; load_image $<; reset; shutdown"

