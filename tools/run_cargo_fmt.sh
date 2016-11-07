#!/usr/bin/env bash

# Find folders with Cargo.toml files in them and run `cargo fmt`.

if [ "$1" == "diff" ]; then
	# Just print out diffs and count errors, used by Travis
	let FAIL=0
	for f in $(find . | grep Cargo.toml); do pushd $(dirname $f); cargo fmt -- --write-mode=diff || let FAIL=FAIL+1; popd; done
	exit $FAIL
else
	for f in $(find . | grep Cargo.toml); do pushd $(dirname $f); cargo fmt -- --write-mode=overwrite; popd; done
fi
