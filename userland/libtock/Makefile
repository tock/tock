MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
CUR_DIR := $(dir $(MKFILE_PATH))

BUILDDIR ?= build/$(ARCH)

TOOLCHAIN ?= arm-none-eabi

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CFLAGS += -Wall -Werror -I$(CUR_DIR) -g -mcpu=$(ARCH) -mthumb
CFLAGS += \
	    -fdata-sections -ffunction-sections\
	    -Wall\
	    -Wextra\
	    -Wl,-gc-sections\
	    -g\
	    -fPIC\
	    -msingle-pic-base\
	    -mpic-register=r9\
	    -mno-pic-data-is-text-relative

AR := $(TOOLCHAIN)-ar

C_SRCS=$(wildcard *.c)
AS_SRCS=syscalls.S
OBJS := $(patsubst %.S,$(BUILDDIR)/%.o,$(AS_SRCS))
OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))

.PHONY: all
all: $(BUILDDIR)/libtock.a

$(BUILDDIR):
	mkdir -p $@

$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(BUILDDIR)/%.o: %.S | $(BUILDDIR)
	$(AS) $(ASFLAGS) -c -o $@ $<

$(BUILDDIR)/libtock.a: $(OBJS) | $(BUILDDIR)
	$(AR) rc $@ $^

.PHONY: clean
clean:
	rm -Rf $(BUILDDIR)

