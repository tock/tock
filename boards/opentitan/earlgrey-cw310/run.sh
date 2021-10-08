#!/bin/bash

if [[ "${OPENTITAN_TREE}" != "" ]]; then
	riscv64-linux-gnu-objcopy --output-target=binary ${1} binary
	${OPENTITAN_TREE}/util/fpga/cw310_loader.py --firmware binary
else
	qemu-system-riscv32 -M opentitan -bios ../../../tools/qemu-runner/opentitan-boot-rom.elf -nographic -serial stdio -monitor none -semihosting -kernel "${1}"
fi

