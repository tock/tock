TOCK_BASE_DIR ?= .
BUILDDIR ?= .
ARCH ?= cortex-m0
LIBTOCK ?= $(TOCK_BASE_DIR)/libtock/build/$(ARCH)/libtock.a

TOOLCHAIN := arm-none-eabi

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(TOCK_BASE_DIR)/tools/elf2tbf/Cargo.toml --

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
	    -mpic-register=r9\
	    -mno-pic-data-is-text-relative

LD := $(TOOLCHAIN)-ld
LINKER ?= $(TOCK_BASE_DIR)/linker.ld
LDFLAGS := -T $(LINKER)

.PHONY:	all
all:	$(BUILDDIR)/app.bin

$(LIBTOCK):
	make -C $(TOCK_BASE_DIR)/libtock ARCH=$(ARCH)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/app.elf: $(OBJS) $(TOCK_BASE_DIR)/newlib/libc.a $(LIBTOCK) | $(BUILDDIR)
	$(LD) --gc-sections --emit-relocs --entry=_start $(LDFLAGS) -nostdlib $(OBJS) --start-group $(TOCK_BASE_DIR)/newlib/libc.a $(LIBTOCK) --end-group -o $@

$(BUILDDIR)/app.bin: $(BUILDDIR)/app.elf | $(BUILDDIR)
	$(ELF2TBF) -o $@ $<

