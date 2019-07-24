#!/usr/bin/env bash

FAIL=0

set -e

# Verify that we're running in the base directory
if [ ! -x tools/run_cargo_fix.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

for f in $(find . | grep Cargo.toml); do
	pushd $(dirname $f) > /dev/null
	cargo fix --allow-dirty || let FAIL=FAIL+1
	popd > /dev/null
done

