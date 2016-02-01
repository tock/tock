$(BUILD_DIR)/arch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S | $(BUILD_DIR)
	@$(TOOLCHAIN)as -mcpu=cortex-m4 -mthumb $^ -o $@
