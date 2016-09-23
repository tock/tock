SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

JLINK=JLinkExe
JLINK_OPTIONS=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS=jtag/

all: build/sam4l/release/storm

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/sam4l/sam4l.json

.PHONY: build/sam4l/release/storm
build/sam4l/release/storm:
	@cargo build --release --target=../../chips/sam4l/sam4l.json
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > build/sam4l/release/storm.lst

build/sam4l/release/storm.sdb: build/sam4l/release/storm
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

target/target/release/storm.hex: target/target/release/storm
	@$(OBJCOPY) -Oihex $^ $@

.PHONY: clean
clean:
	@cargo clean

.PHONY: program
program: build/sam4l/release/storm.sdb
	$(SLOAD) flash $<

.PHONY: flash
flash: target/target/release/storm.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS)flash-kernel.jlink

# special command for the firestorm. Flashes the stormloader bootloader onto
# 	the SAM4L so that sload can communicate with it
.PHONY: flash-bootloader
flash-bootloader: bootloader/bootloader.bin
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS)flash-bootloader.jlink

