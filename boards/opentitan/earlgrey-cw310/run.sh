#!/bin/bash

if [[ "${VERILATOR}" == "yes" ]]; then
	${OBJCOPY} --output-target=binary --strip-sections -S --remove-section .apps "${1}" binary
	srec_cat binary \
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within binary \
		-binary -range-pad 8 --output binary.64.vmem --vmem 64
	${OPENTITAN_TREE}/build/lowrisc_dv_chip_verilator_sim_0.1/sim-verilator/Vchip_sim_tb \
		--meminit=rom,${OPENTITAN_TREE}/build-out/sw/device/boot_rom/boot_rom_sim_verilator.scr.39.vmem \
		--meminit=flash,./binary.64.vmem \
		--meminit=otp,${OPENTITAN_TREE}/build-bin/sw/device/otp_img/otp_img_sim_verilator.vmem
elif [[ "${OPENTITAN_TREE}" != "" ]]; then
	riscv64-linux-gnu-objcopy --update-section .apps=${APP} ${1} bundle.elf
	riscv64-linux-gnu-objcopy --output-target=binary bundle.elf binary
	${OPENTITAN_TREE}/util/fpga/cw310_loader.py --firmware binary
else
	../../../tools/qemu/build/qemu-system-riscv32 -M opentitan -bios ../../../tools/qemu-runner/opentitan-boot-rom.elf -nographic -serial stdio -monitor none -semihosting -kernel "${1}"
fi
