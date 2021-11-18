#!/bin/bash

if [[ "${OPENTITAN_TREE}" != "" ]]; then
	riscv64-linux-gnu-objcopy --update-section .apps=${APP} ${1} bundle.elf
	riscv64-linux-gnu-objcopy --output-target=binary bundle.elf binary
	${OPENTITAN_TREE}/util/fpga/cw310_loader.py --firmware binary
else
	../../../tools/qemu/build/qemu-system-riscv32 -M opentitan -bios ../../../tools/qemu-runner/opentitan-boot-rom.elf -nographic -serial stdio -monitor none -semihosting -kernel "${1}"
fi

