.PHONY: all
all: target/nrf51/release/nrf51dk

.PHONY: doc
doc:
	@cargo doc --release --target=nrf51.json

.PHONY: target/target/release/nrf51dk
target/nrf51/release/nrf51dk:
	@cargo build --release --target=nrf51.json

.PHONY: clean
clean:
	@cargo clean

