LIBFIRESTORM = $(BUILD_DIR)/apps/firestorm.o
LIBTOCK = $(BUILD_DIR)/apps/tock.o

$(LIBFIRESTORM): $(SRC_DIR)apps/libs/firestorm.c $(SRC_DIR)apps/libs/firestorm.h
	@echo "Building libfirestorm"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<

$(LIBTOCK): $(SRC_DIR)apps/libs/tock.c $(SRC_DIR)apps/libs/tock.h
	@echo "Building libtock"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -c -g -Os -o $@ -ffreestanding -nostdlib $<
