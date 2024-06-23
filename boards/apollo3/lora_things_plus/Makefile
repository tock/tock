# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the SparkFun LoRa Thing Plus - expLoRaBLE
#
TARGET=thumbv7em-none-eabi
PLATFORM=lora_things_plus

include ../../Makefile.common

ifndef PORT
  PORT="/dev/ttyUSB0"
endif

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	python ../ambiq/ambiq_bin2board.py --bin $< \
		--load-address-blob 0xE000 \
		-b 115200 -port $(PORT) -r 2 -v \
		--magic-num 0xCB --version 0x0 \
		--load-address-wired 0xC000 \
		-i 6 --options 0x1

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
	python ../ambiq/ambiq_bin2board.py --bin $< \
		--load-address-blob 0xE000 \
		-b 115200 -port $(PORT) -r 2 -v \
		--magic-num 0xCB --version 0x0 \
		--load-address-wired 0xC000 \
		-i 6 --options 0x1

.PHONY: flash-app
flash-app:
	python ../ambiq/ambiq_bin2board.py --bin $(APP) \
		--load-address-blob 0x42000 \
		-b 115200 -port $(PORT) -r 2 -v \
		--magic-num 0xCB --version 0x0 \
		--load-address-wired 0x40000 \
		-i 6 --options 0x1

.PHONY: test
test:
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" OBJCOPY=${OBJCOPY} PORT=$(PORT) $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release
