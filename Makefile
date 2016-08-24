# Makefile for the Tock embedded operating system.
#
# Included Makefiles, in subdirectories, contain most of the build system. See
# individual subdirectories and README for more specific explanation.

# Default platform is the Storm (http://storm.rocks). Change to any platform in
# the `platform` directory.
TOCK_PLATFORM ?= storm
#TOCK_PLATFORM ?= nrf_pca10028

BUILD_ROOT ?= build
BUILD_PLATFORM_DIR ?= $(BUILD_ROOT)/$(TOCK_PLATFORM)

TOCK_ALL_PLATFORMS := $(shell find src/platform -maxdepth 1 -mindepth 1 -type d)
TOCK_ALL_PLATFORMS := $(subst src/platform/,,$(TOCK_ALL_PLATFORMS))

ifeq ($(findstring $(TOCK_PLATFORM),$(TOCK_ALL_PLATFORMS)),)
$(error TOCK_PLATFORM=$(TOCK_PLATFORM) is not in src/platform ?)
endif

# Dummy all. The real one is in platform-specific Makefiles.
all: $(BUILD_ROOT)/elf2tbf

$(BUILD_ROOT):
	@mkdir -p $@

$(BUILD_PLATFORM_DIR): | $(BUILD_ROOT)
	@mkdir -p $@

# Common functions and variables
include Common.mk

BASE_DIR = $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/

# Tools
include tools/Makefile.mk

# External dependencies (Rust libcore)
EXTERN_DIR = $(BASE_DIR)extern/
include extern/Makefile.mk

# Tock
SRC_DIR = $(BASE_DIR)src/
include src/Makefile.mk

.PHONY: doc all clean clean-all

# Generates documentation for the kernel and selected architecture and platform.
doc: $(BUILD_PLATFORM_DIR)/kernel.o
	@echo "Build docs with `make doc` from the platform directory"

# Removes compilation artifacts for Tock, but not external dependencies.
clean:
	rm -rf $(BUILD_PLATFORM_DIR)/*.*

# Remove all compilation artifacts, including external dependencies.
clean-all:
	rm -rf $(BUILD_PLATFORM_DIR)

# Convenience rule that runs `make clean-all` for all valid TOCK_PLATFORMS
clean-all-platforms:
	@for P in $(TOCK_ALL_PLATFORMS); do $(MAKE) --no-print-directory TOCK_PLATFORM=$$P clean-all; done

.PHONY: clean clean-all-platforms clean-all

# Keep all object files
.PRECIOUS: *.o *.elf

