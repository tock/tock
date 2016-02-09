# Support
include $(SRC_DIR)support/Makefile.mk

# Chip
include $(SRC_DIR)chips/Makefile.mk

# Apps --> dependends on Chip to define ARCH
include $(SRC_DIR)apps/Makefile.mk

# Kernel, depends on Chip for ARCH
include $(SRC_DIR)arch/$(ARCH)/Makefile.mk
include $(SRC_DIR)common/Makefile.mk
include $(SRC_DIR)drivers/Makefile.mk
include $(SRC_DIR)hil/Makefile.mk
include $(SRC_DIR)process/Makefile.mk
include $(SRC_DIR)main/Makefile.mk

# Platform --> depends on Apps for APPS
include $(SRC_DIR)platform/Makefile.mk
