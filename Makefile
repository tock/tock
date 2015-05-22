# Makefile for the Tock embedded operating system.
# 
# Included Makfiles, in subdirectories, contain most of the build system. See
# indiviual subdirectories and README for more specific explanation.

BUILD_DIR ?= build

# Default platform is the Storm (http://storm.rocks). Change to any platform in
# the `platform` directory.
PLATFORM ?= storm

# Dummy all. The real one is in platform-specific Makelfes.
all:

$(BUILD_DIR):
	@mkdir -p $@

# Common functions and variables
include Common.mk

# External dependencies (Rust libcore)
include extern/Makefile.mk

# Tock
include src/Makefile.mk

.PHONY: all clean clean-all

# Removes compilation artifacts for Tock, but not external dependencies.
clean:
	rm -Rf $(BUILD_DIR)/*.*

# Remove all compilation artifacts, including for external dependencies.
clean-all:
	rm -Rf $(BUILD_DIR)

