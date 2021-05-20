# Makefile for building the tock kernel for the SweRVolf platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=swervolf

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: sim

setup-sim:
	fusesoc library add fusesoc-cores https://github.com/fusesoc/fusesoc-cores
	fusesoc library add swervolf https://github.com/chipsalliance/Cores-SweRVolf
	fusesoc run --target=sim swervolf

sim: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	python3 makehex.py $^ > binary.hex
	fusesoc run --target=sim --run swervolf --ram_init_file=binary.hex
