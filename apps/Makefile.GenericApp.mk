APP ?= $(notdir $(CURDIR))
APP_DIR ?= $(CURDIR)
APP_LINKER_SCRIPT ?= $(CURDIR)/loader.ld

.SECONDEXPANSION:
.PHONY:	all
all:	$(TOCK_APPS_DIR)/../build/$$(TOCK_PLATFORM)/$(APP)/$(APP).bin.o


include $(TOCK_APPS_DIR)/Makefile.Common.mk

.PHONY:	clean
clean:
	rm -rf $(TOCK_APP_BUILD_DIR)

$(TOCK_APP_BUILD_DIR):
	mkdir -p $@

ifndef TOCK_PLATFORM_LINKER_SCRIPT
$(error TOCK_PLATFORM_LINKER_SCRIPT not defined. Makefile.TOCK_PLATFORM.mk should define this?)
endif

AS_SRCS  := $(wildcard $(APP_DIR)/*.s)
C_SRCS   := $(wildcard $(APP_DIR)/*.c)
CC_SRCS  := $(wildcard $(APP_DIR)/*.cc)
CPP_SRCS := $(wildcard $(APP_DIR)/*.cpp)

LIBS := $(patsubst %.s,%.o,$(AS_SRCS))
LIBS += $(patsubst %.c,%.o,$(C_SRCS))
LIBS += $(patsubst %.cc,%.o,$(CC_SRCS))
LIBS += $(patsubst %.cpp,%.o,$(CPP_SRCS))

LIBS := $(notdir $(LIBS))
LIBS := $(foreach var,$(LIBS),$(TOCK_APP_BUILD_DIR)/$(var))


$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.c | $(TOCK_APP_BUILD_DIR)
	$(CC) $(CFLAGS) $(CPPFLAGS) $^ -c -o $@

$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.cc | $(TOCK_APP_BUILD_DIR)
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) $^ -c -o $@

$(TOCK_APP_BUILD_DIR)/%.o:	$(APP_DIR)/%.cpp | $(TOCK_APP_BUILD_DIR)
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) $^ -c -o $@

# XXX FIXME
$(TOCK_APP_BUILD_DIR)/arch.o:	$(TOCK_DIR)/arch/$(ARCH)/syscalls.S | $(TOCK_APP_BUILD_DIR)
	$(AS) $(ASFLAGS) $^ -o $@

LIBS += $(TOCK_APP_BUILD_DIR)/arch.o

$(TOCK_APP_BUILD_DIR)/$(APP).elf: $(LIBS) $(TOCK_LIBS) $(APP_LIBC) | $(TOCK_APP_BUILD_DIR) kernel
	@echo "Linking $@"
	$(LD) $(CFLAGS) -g -Os -T $(APP_LINKER_SCRIPT) -nostdlib $^ -o $@
	$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).lst

$(TOCK_APP_BUILD_DIR)/$(APP).bin: $(TOCK_APP_BUILD_DIR)/$(APP).elf
	@echo "Extracting binary $@"
	$(OBJCOPY) --gap-fill 0xff -O binary $< $@

$(TOCK_APP_BUILD_DIR)/$(APP).bin.o: $(TOCK_APP_BUILD_DIR)/$(APP).bin
	@echo "Re-Linking $@"
	$(LD) -r -b binary -o $@ $<
	$(OBJCOPY) --rename-section .data=.app.2 $@
	$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(TOCK_APP_BUILD_DIR)/$(APP).bin.lst

APPS_TO_LINK_TO_KERNEL=$(TOCK_APP_BUILD_DIR)/$(APP).bin.o

