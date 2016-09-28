# Makefile for building the tock kernel for the storm platform

SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

SIZE?=arm-none-eabi-size
OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

JLINK=JLinkExe
JLINK_OPTIONS+=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

.PHONY: all
all: target/sam4l/release/storm

.PHONY: doc
doc:
	@cargo doc --release --target=sam4l.json

.PHONY: target/sam4l/release/storm
target/sam4l/release/storm:
	@cargo build --release --target=sam4l.json
	@$(SIZE) $@

.PHONY: target/sam4l/debug/storm
target/sam4l/debug/storm:
	@cargo build --target=sam4l.json
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > target/sam4l/debug/storm.lst
	@$(SIZE) $@

target/sam4l/release/storm.sdb: target/sam4l/release/storm
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

target/sam4l/release/storm.lst: target/sam4l/release/storm
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $^ > $@

target/sam4l/release/storm.hex: target/sam4l/release/storm
	@$(OBJCOPY) -Oihex $^ $@

target/sam4l/debug/storm.hex: target/sam4l/debug/storm
	@$(OBJCOPY) -Oihex $^ $@

.PHONY: clean
clean:
	@cargo clean

.PHONY: debug
debug: target/sam4l/debug/storm

# upload programs over uart with stormloader
.PHONY: program
program: target/target/release/storm.sdb
	$(SLOAD) flash $<

# upload kernel over JTAG
.PHONY: flash
flash: target/sam4l/release/storm.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

.PHONY: flash-debug
flash-debug: target/sam4l/debug/storm.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-debug.jlink

# special command for the firestorm. Flashes the stormloader bootloader onto
# 	the SAM4L so that sload can communicate with it
.PHONY: flash-bootloader
flash-bootloader: bootloader/bootloader.bin
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-bootloader.jlink

