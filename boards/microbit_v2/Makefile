# Makefile for building the tock kernel for the CLUE nRF52840 board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=microbit_v2

include ../Makefile.common

OPENOCD=/home/alex/.arduino15/packages/sandeepmistry/tools/openocd/0.10.0-dev.nrf5/bin/openocd
OPENOCD_OPTIONS=-f openocd.cfg

APP=../../../libtock-c/examples/screen/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) -d2 $(OPENOCD_OPTIONS) -c "program $<; verify_image $<;  reset; shutdown;"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) -d2 $(OPENOCD_OPTIONS) -c "program $<; verify_image $<; reset; shutdown;"

# Upload the kernel using nrfutil
.PHONY: program program-apps

program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
	adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00B6 --application $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin $(KERNEL).zip
	echo "Trying to reset device"
	stty -F $(PORT) 1200 && sleep 0.5 && stty -F $(PORT) 1200
	adafruit-nrfutil --verbose dfu serial -pkg $(KERNEL).zip $(FLAGS) -b 115200 --singlebank

# Upload the kernel and apps using nrfutil
program-apps: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --output-target=binary -S $(KERNEL_WITH_APP) $(KERNEL_WITH_APP).bin
	adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00B6 --application $(KERNEL_WITH_APP).bin $(KERNEL_WITH_APP).zip
	echo "Trying to reset device"
	stty -F $(PORT) 1200 && sleep 0.5 && stty -F $(PORT) 1200
	adafruit-nrfutil --verbose dfu serial -pkg $(KERNEL_WITH_APP).zip $(FLAGS) -b 115200 --singlebank
