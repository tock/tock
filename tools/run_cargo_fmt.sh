#!/usr/bin/env bash

# Find folders with Cargo.toml files in them and run `cargo fmt`.

for f in $(find . | grep Cargo.toml); do pushd $(dirname $f); cargo fmt overwrite; popd; done
