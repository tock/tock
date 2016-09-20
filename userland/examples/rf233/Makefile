CURRENT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

TOCK_BASE_DIR = $(CURRENT_DIR)/../..
BUILDDIR ?= build/$(TOCK_ARCH)

C_SRCS   := $(wildcard *.c)
C_SRCS   := $(wildcard *.c)

OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))

CPPFLAGS += -DSTACK_SIZE=2048

include $(TOCK_BASE_DIR)/Makefile

$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

.PHONY:
clean:
	rm -Rf $(BUILDDIR)
