# Compiler binary defaults. Specific compiler flags in each platform specific
# Makefile.
RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTC_FLAGS += -g -L$(BUILD_PLATFORM_DIR) # Common regardless of platform
CARGO ?= cargo
TOOLCHAIN = arm-none-eabi-
OBJCOPY ?= $(TOOLCHAIN)objcopy
OBJDUMP ?= $(TOOLCHAIN)objdump
CC = $(TOOLCHAIN)gcc
CPP = $(TOOLCHAIN)g++
LD = $(TOOLCHAIN)ld
SIZE = $(TOOLCHAIN)size


# Validate rustc version
# If you change this, you must also change README and .travis.yml
RUSTC_VERSION := $(shell $(RUSTC) --version)
ifneq ($(RUSTC_VERSION),rustc 1.7.0-nightly (110df043b 2015-12-13))
$(warning Tock currently requires rustc version 1.7.0-nightly (110df043b 2015-12-13) exactly)
$(warning Your have $(RUSTC) --version: $(shell $(RUSTC) --version))
$(warning See the README for more information on installing different rustc versions)
$(error Incorrect version of rustc)
endif


# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))


# Default rlib compilation 
.SECONDEXPANSION:
$(BUILD_PLATFORM_DIR)/lib%.rlib: $$(call rwildcard,$(SRC_DIR)$$**/,*.rs) $(BUILD_PLATFORM_DIR)/libcore.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_PLATFORM_DIR) $(SRC_DIR)$*/lib.rs


# Detect currently running OS
# http://stackoverflow.com/questions/714100/os-detecting-makefile
ifeq ($(OS),Windows_NT)
HOST_OS := Windows
else
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
HOST_OS := Linux
endif
ifeq ($(UNAME_S),Darwin)
HOST_OS := Darwin
endif
endif

ifeq ($(HOST_OS),Windows)
TAR ?= tar --force-local
else
TAR ?= tar
endif

