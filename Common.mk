# Compiler binary defaults. Specific compiler flags in each platform specific
# Makefile.
RUSTC ?= rustc
RUSTC_FLAGS += -L$(BUILD_DIR) # Common regardless of platform
TOOLCHAIN = arm-none-eabi-
OBJCOPY ?= $(TOOLCHAIN)objcopy
CC = $(TOOLCHAIN)gcc
CPP = $(TOOLCHAIN)g++
LD = $(TOOLCHAIN)ld

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))

# Default rlib compilation 
.SECONDEXPANSION:
$(BUILD_DIR)/lib%.rlib: $$(call rwildcard,$(SRC_DIR)$$**/,*.rs) $(BUILD_DIR)/libcore.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)$*/lib.rs


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

