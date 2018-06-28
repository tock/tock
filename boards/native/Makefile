#TARGET := $(shell cc -dumpmachine)
TARGET := x86_64-apple-darwin
PLATFORM := tock-native

include ../Makefile.common

override RUSTFLAGS_FOR_CARGO_LINKING := "-Z pre-link-arg=-lSystem"

.PHONY: program
program: target/$(TARGET)/release/$(PLATFORM).bin
	echo "Program not supported."; exit 1

# upload kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).bin
	echo "Flash not supported."; exit 1
