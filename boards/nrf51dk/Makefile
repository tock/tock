# Makefile for building the tock kernel for the nRF development kit

OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

.PHONY: all
all: target/target/release/nrf51dk target/target/release/nrf51dk.lst

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/nrf51/target.json

.PHONY: target/target/release/nrf51dk
target/target/release/nrf51dk:
	@cargo build --release --target=../../chips/nrf51/target.json

target/target/release/nrf51dk.lst: target/target/release/nrf51dk
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $^ > $@

.PHONY: clean
clean:
	@cargo clean

#TODO: Figure out how to program the nRF51dk

