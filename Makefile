# Makefile for the Tock embedded operating system.
# 
# Included Makfiles, in subdirectories, contain most of the build system. See
# indiviual subdirectories and README for more specific explanation.

BUILD_DIR ?= build

# Default platform is the Storm (http://storm.rocks). Change to any platform in
# the `platform` directory.
PLATFORM ?= storm

# Dummy all. The real one is in platform-specific Makefiles.
all:	$(BUILD_DIR)

$(BUILD_DIR):
	@mkdir -p $@/apps

# Common functions and variables
include Common.mk

BASE_DIR = $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/

# External dependencies (Rust libcore)
EXTERN_DIR = $(BASE_DIR)extern/
include extern/Makefile.mk

# Tock
SRC_DIR = $(BASE_DIR)src/
include src/Makefile.mk

.PHONY: all clean clean-all

# Removes compilation artifacts for Tock, but not external dependencies.
clean:
	rm -Rf $(BUILD_DIR)/*.*

# Remove all compilation artifacts, including for external dependencies.
clean-all:
	rm -Rf $(BUILD_DIR)

