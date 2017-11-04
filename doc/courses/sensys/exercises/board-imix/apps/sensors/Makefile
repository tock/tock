# Makefile for user application

TOCK_BOARD=imix

# Specify this directory relative to the current application.
CURRENT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
TOCK_USERLAND_BASE_DIR = $(abspath $(CURRENT_DIR)/../../../../userland)

# Which files to compile.
C_SRCS := $(wildcard *.c)

APP_HEAP_SIZE := 2048

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/Makefile
