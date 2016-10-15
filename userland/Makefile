# userland master makefile. Included by application makefiles

TOCK_USERLAND_BASE_DIR ?= .
TOCK_BASE_DIR ?= ../
BUILDDIR ?= .
TOCK_BOARD ?= storm
TOCK_ARCH ?= cortex-m4
LIBTOCK ?= $(TOCK_USERLAND_BASE_DIR)/libtock/build/$(TOCK_ARCH)/libtock.a

TOOLCHAIN := arm-none-eabi

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(TOCK_USERLAND_BASE_DIR)/tools/elf2tbf/Cargo.toml --

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(TOCK_ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
CPPFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock -g -mcpu=$(TOCK_ARCH) -mthumb -mfloat-abi=soft
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

$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

LD := $(TOOLCHAIN)-ld
LINKER ?= $(TOCK_USERLAND_BASE_DIR)/linker.ld
LDFLAGS := -T $(LINKER)

.PHONY:	all
all:	$(BUILDDIR)/app.bin

$(LIBTOCK):
	make -C $(TOCK_USERLAND_BASE_DIR)/libtock TOCK_ARCH=$(TOCK_ARCH)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/app.elf: $(OBJS) $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) | $(BUILDDIR)
	$(LD) --gc-sections --emit-relocs --entry=_start $(LDFLAGS) -nostdlib $(OBJS) --start-group $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) --end-group -o $@

$(BUILDDIR)/app.bin: $(BUILDDIR)/app.elf | $(BUILDDIR)
	$(ELF2TBF) -o $@ $<

# for programming individual apps, include platform app makefile
#	conditionally included in case it doesn't exist for a board
-include $(TOCK_BASE_DIR)/boards/$(TOCK_BOARD)/Makefile-app

# Include dependency rules for picking up header changes
-include $(OBJS:.o=.d)
