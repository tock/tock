# Makefile for building the tock kernel for the Hail platform

CHIP=sam4l
PLATFORM=hail

include ../../Makefile.common

TOCKLOADER=tockloader

JLINK=JLinkExe
JLINK_OPTIONS+=-device ATSAM4LC8C -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR =jtag

# Where in the SAM4L flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x10000

# Upload programs over uart with tockloader
ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

.PHONY: program
program: target/sam4l/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $<

# upload kernel over JTAG
.PHONY: flash
flash: target/sam4l/release/$(PLATFORM).hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

.PHONY: flash-debug
flash-debug: target/sam4l/debug/$(PLATFORM).hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-debug.jlink

# Command to flash the bootloader. Flashes the bootloader onto the SAM4L.
.PHONY: flash-bootloader
flash-bootloader: bootloader/bootloader.bin
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-bootloader.jlink

# Command to flash the nRF51822 on Hail with the connectivity side
# of the BLE serialization library.

# The ID is parsed from the command line and written to a particular address in
# flash .
ifdef ID
  # Write the ID to flash as well
  ID_BYTES = $(subst :, ,$(ID))
  NUM_ID_BYTES = $(words $(ID_BYTES))
  ifneq ($(NUM_ID_BYTES),6)
    $(error "Invalid number of bytes in ID string (expecting 6)")
  endif
  ID_FIRST = $(word 1,$(ID_BYTES))$(word 2,$(ID_BYTES))
  ID_SECON = $(word 3,$(ID_BYTES))$(word 4,$(ID_BYTES))$(word 5,$(ID_BYTES))$(word 6,$(ID_BYTES))
  BLEADDR_FLASH_LOCATION ?= 0x0003FFF8
endif

.PHONY: flash-nrf51822
flash-nrf51822: nrf51822/tock-nrf51822-serialization-sdk11-s130-uart-conn-hail.hex
	printf "r\n" > $(JLINK_SCRIPTS_DIR)/flash-nrf51822.jlink
ifdef ID
	printf "w4 $(BLEADDR_FLASH_LOCATION), 0x$(ID_SECON) 0x$(ID_FIRST)\n" >> $(JLINK_SCRIPTS_DIR)/flash-nrf51822.jlink
endif
	printf "loadfile $< \nr\ng\nexit\n" >> $(JLINK_SCRIPTS_DIR)/flash-nrf51822.jlink
	$(JLINK) -device nrf51822 -if swd -speed 1000 $(JLINK_SCRIPTS_DIR)/flash-nrf51822.jlink
