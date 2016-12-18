# Makefile for building the tock kernel for the Hail platform

TOCKLOADER=tockloader
SIZE?=arm-none-eabi-size
OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

JLINK=JLinkExe
JLINK_OPTIONS+=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR =jtag

PLATFORM=hail

# Where in the SAM4L flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x10000

.PHONY: all
all: target/sam4l/release/$(PLATFORM).bin

.PHONY: doc
doc:
	@cargo doc --release --target=sam4l.json

.PHONY: target/sam4l/release/$(PLATFORM).elf
target/sam4l/release/$(PLATFORM).elf:
	@cargo build --release --target=sam4l.json
	@cp target/sam4l/release/$(PLATFORM) target/sam4l/release/$(PLATFORM).elf
	@$(SIZE) $@

.PHONY: target/sam4l/debug/$(PLATFORM).elf
target/sam4l/debug/$(PLATFORM).elf:
	@cargo build --target=sam4l.json
	@cp target/sam4l/debug/$(PLATFORM) target/sam4l/debug/$(PLATFORM).elf
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > target/sam4l/debug/$(PLATFORM).lst
	@$(SIZE) $@

target/sam4l/release/$(PLATFORM).hex: target/sam4l/release/$(PLATFORM).elf
	@$(OBJCOPY) -Oihex $^ $@

target/sam4l/debug/$(PLATFORM).hex: target/sam4l/debug/$(PLATFORM).elf
	@$(OBJCOPY) -Oihex $^ $@

target/sam4l/release/$(PLATFORM).bin: target/sam4l/release/$(PLATFORM).elf
	@$(OBJCOPY) -Obinary $^ $@

target/sam4l/debug/$(PLATFORM).bin: target/sam4l/debug/$(PLATFORM).elf
	@$(OBJCOPY) -Obinary $^ $@

.PHONY: clean
clean::
	@cargo clean

.PHONY: debug
debug: target/sam4l/debug/$(PLATFORM).elf

# upload programs over uart with $(PLATFORM)loader
.PHONY: program
program: target/sam4l/release/$(PLATFORM).bin
	$(TOCKLOADER) flash --address $(KERNEL_ADDRESS) $<

# upload kernel over JTAG
.PHONY: flash
flash: target/sam4l/release/$(PLATFORM).hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

.PHONY: flash-debug
flash-debug: target/sam4l/debug/$(PLATFORM).hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-debug.jlink

# special command for the fire$(PLATFORM). Flashes the $(PLATFORM)loader bootloader onto
# 	the SAM4L so that sload can communicate with it
.PHONY: flash-bootloader
flash-bootloader: bootloader/bootloader.bin
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-bootloader.jlink

