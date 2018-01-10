# Makefile for user applications

# Specify this directory relative to the current application.
# NOTE: This should be the only line, if any, that you need to change to build
# basic C applications.
include ../../MakeVariables.mk

# Which files to compile.
C_SRCS := $(wildcard *.c)

# Include libteensy header files
override CPPFLAGS += -I../../../libteensy
EXTERN_LIBS += ../../../libteensy

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/AppMakefile.mk

APP=$(patsubst $(APPS_DIR),,$(CURDIR))
