# Makefile for user application

# Specify this directory relative to the current application.
TOCK_USERLAND_BASE_DIR = ../..

# Which files to compile.
C_SRCS := $(wildcard *.c)

# Include the makefile for using BLE serialization.
include $(TOCK_USERLAND_BASE_DIR)/libnrfserialization/Makefile.app

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/Makefile
