# Makefile for building the tock kernel for the i.MX RT1052 platform
# 
APP=../../../libtock-c/examples/c_hello/build/cortex-m7/cortex-m7.tbf
# APP=../../../libtock-rs/target/thumbv7em-none-eabi/tab/imxrt1050/hello_world/cortex-m7.tbf 
TARGET=thumbv7em-none-eabi
PLATFORM=imxrt1050-evkb

D_KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
D_KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.axf
R_KERNEL=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
R_KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.axf

MCUX_IDE_BIN=/Applications/MCUXpressoIDE_11.1.1_3241/ide/plugins/com.nxp.mcuxpresso.tools.bin.macosx_11.1.0.202002241259/binaries/

include ../Makefile.common

.PHONY: $(TOCK_ROOT_DIRECTORY)program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(D_KERNEL) $(D_KERNEL_WITH_APP)
	$(warning Tock OS was compiled. In order to flash it on the board, you will need MCUExpresso. See README.md for details.)

.PHONY: $(TOCK_ROOT_DIRECTORY)flash-debug
flash-debug: program
	$(MCUX_IDE_BIN)/crt_emu_cm_redlink --flash-load-exec "$(D_KERNEL_WITH_APP)" -p MIMXRT1052xxxxB \
		--ConnectScript RT1050_connect.scp --flash-driver= -x . \
		--flash-dir $(MCUX_IDE_BIN)/Flash --flash-hashing

.PHONY: $(TOCK_ROOT_DIRECTORY)flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(R_KERNEL) $(R_KERNEL_WITH_APP)
	$(MCUX_IDE_BIN)/crt_emu_cm_redlink --flash-load-exec "$(R_KERNEL_WITH_APP)" -p MIMXRT1052xxxxB \
		--ConnectScript RT1050_connect.scp --flash-driver= -x . \
		--flash-dir $(MCUX_IDE_BIN)/Flash --flash-hashing
