# Makefile for building the tock kernel for the Arduino Nano 33 BLE board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=clue_nrf52840

include ../Makefile.common

ifdef PORT
  FLAGS += -p $(PORT)
endif

APP=../../../libtock-c/examples/buttons/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

# Upload the kernel using bossac
.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --output-target=binary -S $(KERNEL_WITH_APP) $(KERNEL_WITH_APP).bin
	adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00B6 --application $(KERNEL_WITH_APP).bin $(KERNEL_WITH_APP).zip
	adafruit-nrfutil --verbose dfu serial -pkg $(KERNEL_WITH_APP).zip $(FLAGS) -b 115200 --singlebank
