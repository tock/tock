# Makefile for building the tock kernel for the WeAct STM32F401CCU6 Core Board
TARGET=thumbv7em-none-eabihf
PLATFORM=weact-f401ccu6
KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f openocd.cfg

flash: $(KERNEL)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

flash-app: $(KERNEL)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $< $(KERNEL_WITH_APP)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
