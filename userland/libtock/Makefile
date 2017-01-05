LIBTOCK_DIR := $(TOCK_USERLAND_BASE_DIR)/libtock

ifeq ($(LIBTOCK_DIR),/libtock)
$(error Do not run make from the libtock directory)
endif

LIBTOCK_BUILDDIR ?= $(LIBTOCK_DIR)/build/$(TOCK_ARCH)

TOOLCHAIN ?= arm-none-eabi

CFLAGS += -I$(LIBTOCK_DIR)

AR := $(TOOLCHAIN)-ar

LIBTOCK_C_SRCS=$(wildcard $(LIBTOCK_DIR)/*.c)
LIBTOCK_AS_SRCS=$(wildcard $(LIBTOCK_DIR)/*.s)
LIBTOCK_OBJS := $(patsubst $(LIBTOCK_DIR)/%.s,$(LIBTOCK_BUILDDIR)/%.o,$(LIBTOCK_AS_SRCS))
LIBTOCK_OBJS += $(patsubst $(LIBTOCK_DIR)/%.c,$(LIBTOCK_BUILDDIR)/%.o,$(LIBTOCK_C_SRCS))

.PHONY: all
all: $(LIBTOCK_BUILDDIR)/libtock.a

$(LIBTOCK_BUILDDIR):
	$(Q)mkdir -p $@

$(LIBTOCK_BUILDDIR)/%.o: $(LIBTOCK_DIR)/%.c | $(LIBTOCK_BUILDDIR)
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

$(LIBTOCK_BUILDDIR)/%.o: %.S | $(LIBTOCK_BUILDDIR)
	$(TRACE_AS)
	$(Q)$(AS) $(ASFLAGS) $(CPPFLAGS) -c -o $@ $<

$(LIBTOCK_BUILDDIR)/libtock.a: $(LIBTOCK_OBJS) | $(LIBTOCK_BUILDDIR)
	$(TRACE_AR)
	$(Q)$(AR) rc $@ $^
	$(Q)$(TOOLCHAIN)-ranlib $@

.PHONY: clean
clean::
	rm -Rf $(LIBTOCK_BUILDDIR)
