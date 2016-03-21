$(BUILD_PLATFORM_DIR)/ctx_switch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S | $(BUILD_PLATFORM_DIR)
	@$(TOOLCHAIN)as -mcpu=cortex-m4 -mthumb $^ -o $@
