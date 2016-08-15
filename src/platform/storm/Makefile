SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

all: target/target/release/storm

.PHONY: target/target/release/storm
target/target/release/storm:
	@cargo build --release --target=../../chips/sam4l/target.json

target/target/release/storm.sdb: target/target/release/storm
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: clean
clean:
	@cargo clean

.PHONY: program
program: target/target/release/storm.sdb
	$(SLOAD) flash $<

