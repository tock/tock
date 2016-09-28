# Makefile for building the tock kernel for the nRF development kit

SIZE?=arm-none-eabi-size
OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

.PHONY: all
all: target/nrf51/release/nrf51dk

.PHONY: doc
doc:
	@cargo doc --release --target=nrf51.json

.PHONY: target/nrf51/release/nrf51dk
target/nrf51/release/nrf51dk:
	@cargo build --release --target=nrf51.json
	@$(SIZE) $@

.PHONY: target/nrf51/debug/nrf51dk
target/nrf51/debug/nrf51dk:
	@cargo build --target=nrf51.json
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > target/nrf51/debug/nrf51dk.lst
	@$(SIZE) $@

.PHONY: clean
clean:
	@cargo clean

.PHONY: debug
debug: target/nrf51/debug/nrf51dk

#TODO: Figure out how to program the nRF51dk

