#!/usr/bin/env bash

# Peg a rustfmt version while things are unstable
#
# Note: We install a local copy of rustfmt so as not to interfere with any
# other use of rustfmt on the machine
RUSTFMT_VERSION=0.7.1


# Verify that we're running in the base directory
if [ ! -x tools/run_cargo_fmt.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

# CI setup has correct rustfmt install globally already
if [ ! "$CI" == "true" ]; then
	# Check to make sure that cargo format is installed
	if [ ! -x tools/local_cargo/bin/rustfmt ]; then
		echo "INFO: rustfmt v$RUSTFMT_VERSION not installed. Installing."
		echo "(This will take a few minutes)"
		echo ""
		mkdir -p tools/local_cargo
		cargo install --root tools/local_cargo --vers $RUSTFMT_VERSION --force rustfmt
	fi
	# Put local cargo format on PATH
	PATH="$(pwd)/tools/local_cargo/bin:$PATH"
fi

# Find folders with Cargo.toml files in them and run `cargo fmt`.
if [ "$1" == "diff" ]; then
	# Just print out diffs and count errors, used by Travis
	let FAIL=0
	for f in $(find . | grep Cargo.toml); do
		pushd $(dirname $f) > /dev/null
		cargo-fmt -- --write-mode=diff || let FAIL=FAIL+1
		popd > /dev/null
	done
	exit $FAIL
else
	for f in $(find . | grep Cargo.toml); do
		pushd $(dirname $f) > /dev/null
		cargo-fmt -- --write-mode=overwrite
		popd > /dev/null
	done
fi
