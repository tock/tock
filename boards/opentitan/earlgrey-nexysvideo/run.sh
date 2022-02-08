#!/bin/bash

BUILD_DIR="verilator_build/"

if [[ "${VERILATOR}" == "yes" ]]; then
		if [ -d "$BUILD_DIR" ]; then
			# Cleanup before we build again
			printf "\n[NexysVideo: Verilator Tests] Cleaning up verilator_build...\n\n"
			rm -R "$BUILD_DIR"/*
		else
			printf "\n[NexysVideo: Verilator Tests] Setting up verilator_build...\n\n"
			mkdir "$BUILD_DIR"
		fi
	# Copy in and covert from cargo test output to binary
	${OBJCOPY} ${1} "$BUILD_DIR"/earlgrey-cw310-tests.elf
	${OBJCOPY} --output-target=binary "$BUILD_DIR"/earlgrey-cw310-tests.elf "$BUILD_DIR"/earlgrey-cw310-tests.bin
	# Create VMEM file from test binary
	srec_cat "$BUILD_DIR"/earlgrey-cw310-tests.bin\
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within "$BUILD_DIR"/earlgrey-cw310-tests.bin\
		-binary -range-pad 8 --output "$BUILD_DIR"/binary.64.vmem --vmem 64
	${OPENTITAN_TREE}/build/lowrisc_dv_chip_verilator_sim_0.1/sim-verilator/Vchip_sim_tb \
		--meminit=rom,${OPENTITAN_TREE}/build-out/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
		--meminit=flash,./"$BUILD_DIR"/binary.64.vmem \
		--meminit=otp,${OPENTITAN_TREE}/build-out/sw/device/otp_img/otp_img_sim_verilator.vmem
elif [[ "${OPENTITAN_TREE}" != "" ]]; then
	${OBJCOPY} --output-target=binary ${1} binary
	${OPENTITAN_TREE}/build-out/sw/host/spiflash/spiflash --dev-id=0403:6010 --input=binary
else
	../../../tools/qemu/build/qemu-system-riscv32 -M opentitan -bios ../../../tools/qemu-runner/opentitan-boot-rom.elf -nographic -serial stdio -monitor none -semihosting -kernel "${1}"
fi
