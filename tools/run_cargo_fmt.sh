#!/usr/bin/env bash

# Check to make sure that cargo format is installed
cargo --list | grep fmt > /dev/null
let rc=$?
if [[ $rc != 0 ]]; then
	echo ERROR: rustfmt is not installed.
	echo Run \`cargo install rustfmt\`
	echo ""
	exit 1
fi

# Find folders with Cargo.toml files in them and run `cargo fmt`.

if [ "$1" == "diff" ]; then
	# Just print out diffs and count errors, used by Travis
	let FAIL=0
	for f in $(find . | grep Cargo.toml); do
		pushd $(dirname $f) > /dev/null
		cargo fmt -- --write-mode=diff || let FAIL=FAIL+1
		popd > /dev/null
	done
	exit $FAIL
else
	for f in $(find . | grep Cargo.toml); do
		pushd $(dirname $f) > /dev/null
		cargo fmt -- --write-mode=overwrite
		popd > /dev/null
	done
fi
