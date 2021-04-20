# Makefile for building the tock kernel for the ESP32-C3 platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=esp32-c3-board

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	esptool.py --port /dev/ttyUSB0 --chip esp32c3 elf2image --use_segments --output binary.hex $^
	esptool.py --port /dev/ttyUSB0 --chip esp32c3 write_flash --flash_mode dio --flash_size detect --flash_freq 80m  0x0 binary.hex
