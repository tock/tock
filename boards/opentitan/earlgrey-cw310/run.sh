#!/bin/bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

BUILD_DIR="verilator_build/"

if [[ "${VERILATOR}" == "yes" ]]; then
		if [ -d "$BUILD_DIR" ]; then
			# Cleanup before we build again
			printf "\n[CW-130: Verilator Tests]: Cleaning up verilator_build...\n\n"
			rm -R "$BUILD_DIR"/*
		else
			printf "\n[CW-130: Verilator Tests]: Setting up verilator_build...\n\n"
			mkdir "$BUILD_DIR"
		fi
	# Copy in and covert from cargo test output to binary
	${OBJCOPY} ${1} "$BUILD_DIR"/earlgrey-cw310-tests.elf
	if [[ "${APP}" != "" ]]; then
		# An app was specified, copy it in
		printf "[CW-130: Verilator Tests]: Linking APP\n\n"
		${OBJCOPY} --update-section .apps=${APP} "$BUILD_DIR"/earlgrey-cw310-tests.elf "$BUILD_DIR"/earlgrey-cw310-tests.elf
	fi
	${OBJCOPY} --output-target=binary "$BUILD_DIR"/earlgrey-cw310-tests.elf "$BUILD_DIR"/earlgrey-cw310-tests.bin
	# Create VMEM file from test binary
	srec_cat "$BUILD_DIR"/earlgrey-cw310-tests.bin\
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within "$BUILD_DIR"/earlgrey-cw310-tests.bin\
		-binary -range-pad 8 --output "$BUILD_DIR"/binary.64.vmem --vmem 64
	${OPENTITAN_TREE}/bazel-bin/hw/build.verilator_real/sim-verilator/Vchip_sim_tb \
		--meminit=rom,${OPENTITAN_TREE}/bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.39.scr.vmem \
		--meminit=flash0,./"$BUILD_DIR"/binary.64.vmem \
		--meminit=otp,${OPENTITAN_TREE}/bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/img_rma.24.vmem
elif [[ "${OPENTITAN_TREE}" != "" ]]; then
	${OBJCOPY} --update-section .apps=${APP} ${1} bundle.elf
	${OBJCOPY} --output-target=binary bundle.elf binary

	${OPENTITAN_TREE}/bazel-bin/sw/host/opentitantool/opentitantool.runfiles/lowrisc_opentitan/sw/host/opentitantool/opentitantool --interface=cw310 bootstrap binary
else
	../../../tools/ci/qemu/build/qemu-system-riscv32 -M opentitan -nographic -serial stdio -monitor none -semihosting -kernel "${1}" -global driver=riscv.lowrisc.ibex.soc,property=resetvec,value=${QEMU_ENTRY_POINT}
fi
