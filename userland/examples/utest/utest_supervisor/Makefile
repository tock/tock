# Makefile for user application

PACKAGE_NAME="org.tockos.utest"

# Specify this directory relative to the current application.
TOCK_USERLAND_BASE_DIR = ../../..

# External libraries used
EXTERN_LIBS += $(TOCK_USERLAND_BASE_DIR)/libutest

# Which files to compile.
C_SRCS := $(wildcard *.c)

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/AppMakefile.mk
