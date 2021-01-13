#!/usr/bin/env bash

# Find folders with Cargo.toml files but no README.md file.
for f in $(find . | grep Cargo.toml); do
	dir=$(dirname $f)
	readme=${dir}/README.md

	if [ ! -f "$readme" ]; then
	    echo "$readme does not exist!"
	fi
done
