APPS_DIR := $(dir $(lastword $(MAKEFILE_LIST)))
TOCK_USERLAND_BASE_DIR := $(APPS_DIR)../tock/userland
