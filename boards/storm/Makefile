SLOAD=sload
SDB=$(TOCK_BUILD_DIR)/kernel.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

all: build/sam4l/release/storm

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/sam4l/sam4l.json

.PHONY: build/sam4l/release/storm
build/sam4l/release/storm:
	@cargo build --release --target=../../chips/sam4l/sam4l.json

build/sam4l/release/storm.sdb: build/sam4l/release/storm
	@tput bold ; echo "Packing SDB..." ; tput sgr0
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: clean
clean:
	@cargo clean

.PHONY: program
program: build/sam4l/release/storm.sdb
	$(SLOAD) flash $<

