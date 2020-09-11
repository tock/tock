# Makefile for building the tock kernel for the RedBoard Artemis Nano platform
#
TARGET=thumbv7em-none-eabi
PLATFORM=redboard_artemis_nano

include ../Makefile.common

ifndef PORT
  PORT="/dev/ttyUSB0"
endif

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	ambiq/ambiq_bin2board.py --bin $< --load-address-blob 0x20000 -b 115200 -port $(PORT) -r 2 -v --magic-num 0xCB --version 0x0 --load-address-wired 0xc000 -i 6 --options 0x1

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
	ambiq/ambiq_bin2board.py --bin $< --load-address-blob 0x20000 -b 115200 -port $(PORT) -r 2 -v --magic-num 0xCB --version 0x0 --load-address-wired 0xc000 -i 6 --options 0x1
