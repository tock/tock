# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=hifive1

include ../Makefile.common

flash: target/$(TARGET)/release/$(PLATFORM).elf
	openocd \
		-c "source [find board/sifive-hifive1.cfg]; flash protect 0 64 last off; program $<; resume 0x20400000; exit"


# TOCKLOADER=tockloader

# # Where in the SAM4L flash to load the kernel with `tockloader`
# KERNEL_ADDRESS=0x10000

# # Upload programs over uart with tockloader
# ifdef PORT
#   TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
# endif

# TOCKLOADER_JTAG_FLAGS = --jtag --board hail --arch cortex-m4 --jtag-device ATSAM4LC8C

# .PHONY: program
# program: target/$(TARGET)/release/$(PLATFORM).bin
# 	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $<

# # upload kernel over JTAG
# .PHONY: flash
# flash: target/$(TARGET)/release/$(PLATFORM).bin
# 	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

# .PHONY: flash-debug
# flash-debug: target/$(TARGET)/debug/$(PLATFORM).bin
# 	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

# # Command to flash the bootloader. Flashes the bootloader onto the SAM4L.
# .PHONY: flash-bootloader
# flash-bootloader: bootloader/bootloader.bin
# 	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address 0 $(TOCKLOADER_JTAG_FLAGS) $<
