$(BUILD_DIR)/arch.o: $(SRC_DIR)arch/cortex-m4/ctx_switch.S $(SRC_DIR)arch/cortex-m4/syscalls.S | $(BUILD_DIR)
	@$(TOOLCHAIN)as -mcpu=cortex-m4 -mthumb $^ -o $@
