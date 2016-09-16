TOCK_BASE_DIR ?= .
BUILDDIR ?= .

TOOLCHAIN := arm-none-eabi

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
CPPFLAGS += -I$(TOCK_BASE_DIR)/libtock -g -mcpu=$(ARCH) -mthumb -mfloat-abi=soft
CPPFLAGS += \
	    -fdata-sections -ffunction-sections\
	    -Wall\
	    -Wextra\
	    -Wl,-gc-sections\
	    -g\
	    -fPIC\
	    -msingle-pic-base\
	    -mno-pic-data-is-text-relative

LD := $(TOOLCHAIN)-ld
LINKER ?= $(TOCK_BASE_DIR)/linker.ld
LDFLAGS := -T $(LINKER)

# Include an all target at the top so that all becomes the default goal
#
# Note that this makefile only gets as far as building/requiring the application
# image that will be loaded into tock. The platform makefile provides further
# dependencies to the all target such that a unified kernel+app image is built
#
# This makefile has rules to create up to $(APP).elf, the Common rules convert
# a built application to the linkable monolithic object given as a target here
.SECONDEXPANSION:
.PHONY:	all
all:	$(BUILDDIR)/app.elf

.PHONY:	clean
clean:
	rm -Rf build/$(ARCH)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/stage0.elf: $(OBJS) $(LIBTOCK) $(TOCK_BASE_DIR)/newlib/libc.a | $(BUILDDIR)
	$(LD) --gc-sections --entry=_start $(LDFLAGS) -nostdlib $^ -o $@

$(BUILDDIR)/app.elf: $(BUILDDIR)/stage0.elf | $(BUILDDIR)
	$(LD) -Os $(LDFLAGS) --emit-relocs -nostdlib $^ -o $@

