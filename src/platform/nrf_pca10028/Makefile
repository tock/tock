.PHONY: all
all: target/target/release/nrf_pca10028

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/nrf51/target.json

.PHONY: target/target/release/nrf_pca10028
target/target/release/nrf_pca10028:
	@cargo build --release --target=../../chips/nrf51/target.json

.PHONY: clean
clean:
	@cargo clean

