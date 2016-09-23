.PHONY: all
all: build/nrf51/release/nrf_pca10028

.PHONY: doc
doc:
	@cargo doc --release --target=../../chips/nrf51/nrf51.json

.PHONY: build/nrf51/release/nrf_pca10028
build/nrf51/release/nrf_pca10028:
	@cargo build --release --target=../../chips/nrf51/nrf51.json

.PHONY: clean
clean:
	@cargo clean

