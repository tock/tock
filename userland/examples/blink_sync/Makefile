C_SRCS   := $(wildcard *.c)
C_SRCS   := $(wildcard *.c)

OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))

$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) $< -c -o $@


CPPFLAGS += -DSTACK_SIZE=2048

TOCK_BASE_DIR = ../..
LIBTOCK ?= $(TOCK_BASE_DIR)/libtock/build/cortex-m4/libtock.a
BUILDDIR ?= build/$(ARCH)

include $(TOCK_BASE_DIR)/Makefile

$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<
