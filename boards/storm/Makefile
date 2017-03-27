# Makefile for building the tock kernel for the storm platform

CHIP     := sam4l
PLATFORM := storm

include ../Makefile.common

SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK=JLinkExe
JLINK_OPTIONS+=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

target/sam4l/release/storm.sdb: target/sam4l/release/storm
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

# upload programs over uart with stormloader
.PHONY: program
program: target/sam4l/release/storm.sdb
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

