.PHONY: all
all:


TOCK_USERLAND_BASE_DIR ?= ../tock/userland

LIBNAME=libteensy
$(LIBNAME)_DIR=.
$(LIBNAME)_SRCS=$(wildcard $($(LIBNAME)_DIR)/*.c)

override CPPFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock

include $(TOCK_USERLAND_BASE_DIR)/TockLibrary.mk
