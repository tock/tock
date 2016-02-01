$(BUILD_DIR)/arch.o: $(SRC_DIR)arch/$(ARCH)/ctx_switch.S
	@$(TOOLCHAIN)as -mcpu=$(ARCH) -mthumb $^ -o $@
