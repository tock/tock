# Makefile for building the tock kernel for the nRF development kit

SIZE?=arm-none-eabi-size
OBJCOPY?=arm-none-eabi-objcopy
OBJDUMP?=arm-none-eabi-objdump
OBJDUMP_FLAGS+= --disassemble-all --source --disassembler-options=force-thumb -C --section-headers

TOCK_ARCH=cortex-m0

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

.PHONY: apps/$(APP)/build/$(TOCK_ARCH)/app.bin
apps/$(APP)/build/$(TOCK_ARCH)/app.bin:
	@make -C apps/$(APP) TOCK_ARCH=$(TOCK_ARCH)

target/nrf51/release/nrf51dk-$(APP): target/nrf51/release/nrf51dk apps/$(APP)/build/$(TOCK_ARCH)/app.bin
	@$(OBJCOPY) --update-section .apps=../../userland/examples/$(APP)/build/$(TOCK_ARCH)/app.bin \
	  --set-section-flags .apps=alloc,code \
	  target/nrf51/release/nrf51dk $@

target/nrf51/release/nrf51dk.hex: target/nrf51/release/nrf51dk
	@$(OBJCOPY) -Oihex $^ $@

target/nrf51/release/nrf51dk-$(APP).hex: target/nrf51/release/nrf51dk-$(APP)
	@$(OBJCOPY) -Oihex $^ $@

JLINK=JLinkExe
JLINK_OPTIONS+=-device nrf51422 -if swd -speed 1200 -AutoConnect 1
JLINK_SCRIPTS_DIR=jtag/

# Upload the kernel over JTAG
.PHONY: program
program: target/nrf51/release/nrf51dk.hex
	$(JLINK) $(JLINK_OPTIONS) $(JLINK_SCRIPTS_DIR)/flash-kernel.jlink

# Upload kernel + app over JTAG
.PHONY: program-full
program-full: target/nrf51/release/nrf51dk-$(APP).hex
	@$(eval TEMPFILE := $(shell mktemp -t nrf51dk-$(APP).jlink.XXXXXXXXXX))
	@echo r > $(TEMPFILE)
	@echo loadfile $< >> $(TEMPFILE)
	@echo r >> $(TEMPFILE)
	@echo g >> $(TEMPFILE)
	@echo q >> $(TEMPFILE)
	$(JLINK) $(JLINK_OPTIONS) $(TEMPFILE)
	@rm $(TEMPFILE)

