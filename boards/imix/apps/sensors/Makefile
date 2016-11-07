# makefile for user application

TOCK_BOARD=imix

CURRENT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

TOCK_BASE_DIR = $(abspath $(CURRENT_DIR)/../../../../)
TOCK_USERLAND_BASE_DIR = $(TOCK_BASE_DIR)/userland
BUILDDIR ?= $(abspath build/$(TOCK_ARCH))


C_SRCS   := $(wildcard *.c)
OBJS += $(patsubst %.c,$(BUILDDIR)/%.o,$(C_SRCS))

CPPFLAGS += -DSTACK_SIZE=2048

# include userland master makefile. Contains rules and flags for actually
# 	building the application
include $(TOCK_USERLAND_BASE_DIR)/Makefile

.PHONY:
clean::
	rm -Rf $(BUILDDIR)

