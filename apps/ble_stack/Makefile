TOCK_APPS_DIR = ..
APP           = ble_stack
CFLAGS        = -Iinclude
#APPLICATION_SRCS = $(notdir $(wildcard/*.c))
#APPLICATION_SRCS = ble_phy.c

include $(TOCK_APPS_DIR)/Makefile.GenericApp.mk

CPPFLAGS += -DSTACK_SIZE=2048
