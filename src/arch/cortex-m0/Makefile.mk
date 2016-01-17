$(BUILD_DIR)/arch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S $(SRC_DIR)arch/$(ARCH)/syscalls.S
	@$(TOOLCHAIN)as -mcpu=$(ARCH) -mthumb $^ -o $@
