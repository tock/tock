# Makefile for building the tock kernel for the Arduino Nano 33 BLE board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=nano33ble

include ../Makefile.common

ifdef PORT
  FLAGS += --port $(PORT)
endif

# Upload the kernel using tockloader and the tock bootloader
.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader $(FLAGS) flash --address 0x10000 $<
