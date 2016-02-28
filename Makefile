# Makefile for the Tock embedded operating system.
#
# Included Makfiles, in subdirectories, contain most of the build system. See
# indiviual subdirectories and README for more specific explanation.

# Default platform is the Storm (http://storm.rocks). Change to any platform in
# the `platform` directory.
TOCK_PLATFORM ?= storm

BUILD_ROOT ?= build
BUILD_PLATFORM_DIR ?= $(BUILD_ROOT)/$(TOCK_PLATFORM)

# Dummy all. The real one is in platform-specific Makefiles.
all:	$(BUILD_ROOT) $(BUILD_PLATFORM_APP_DIR)

$(BUILD_ROOT):
	@mkdir -p $@

$(BUILD_PLATFORM_DIR): $(BUILD_ROOT)
	@mkdir -p $@

#$(BUILD_PLATFORM_APP_DIR): $(BUILD_PLATFORM_DIR)
#	@mkdir -p $@

# Common functions and variables
include Common.mk

BASE_DIR = $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/

# External dependencies (Rust libcore)
EXTERN_DIR = $(BASE_DIR)extern/
include extern/Makefile.mk

# Tock
SRC_DIR = $(BASE_DIR)src/
include src/Makefile.mk

.PHONY: doc all clean clean-all

# Generates documentation for the kernel and selected architecture and platform.
doc: $(BUILD_PLATFORM_DIR)/main.o
	@echo "Generating documentation..."
	# Break this temporarily; we'll fix later when this is done recursively
	#@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)chips/$(CHIP)/lib.rs
	@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)common/lib.rs
	@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)drivers/lib.rs
	@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)hil/lib.rs
	@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)main/main.rs
	@$(RUSTDOC) --target $(RUST_TARGET) -L$(BUILD_PLATFORM_DIR) $(SRC_DIR)platform/$(TOCK_PLATFORM)/lib.rs

# Removes compilation artifacts for Tock, but not external dependencies.
clean:
	rm -Rf $(BUILD_PLATFORM_DIR)/*.*

# Remove all compilation artifacts, including for external dependencies.
clean-all:
	rm -Rf $(BUILD_PLATFORM_DIR)

# Keep all object files
.PRECIOUS: *.o *.elf

