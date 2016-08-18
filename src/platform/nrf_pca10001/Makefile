.PHONY: all
all: target/target/release/nrf_pca10001

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/sam4l/target.json

.PHONY: target/target/release/nrf_pca10001
target/target/release/nrf_pca10001:
	@cargo build --release --target=../../chips/nrf51822/target.json

.PHONY: clean
clean:
	@cargo clean

