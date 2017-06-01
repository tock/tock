# Makefile for user application

# Specify this directory relative to the current application.
TOCK_USERLAND_BASE_DIR = ../..

# Which files to compile.
C_SRCS := $(wildcard *.c)

# External libraries used
EXTERN_LIBS += $(TOCK_USERLAND_BASE_DIR)/lua53

override CFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/lua53/include

STACK_SIZE    = 3072
KERNEL_HEAP_SIZE    = 512
APP_HEAP_SIZE    = 9216

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/AppMakefile.mk
