#!/usr/bin/env bash

# Find folders with Cargo.toml files in them and run `cargo fmt`.

for f in $(find -path ./extern -prune -o -name Cargo.toml -print); do pushd ${f%/Cargo.toml}; cargo fmt; popd; done
