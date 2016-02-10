APPS ?= c_blinky
#APPS ?= spi_buf

APP_LIBC := extern/newlib/libc.a
CFLAGS_APPS := -I$(SRC_DIR)apps/libs -fPIC -msingle-pic-base -mno-pic-data-is-text-relative

include $(SRC_DIR)apps/*/Makefile.mk

APP_BINS = $(foreach app,$(APPS),$(BUILD_APP_DIR)/$(app).bin.o)

