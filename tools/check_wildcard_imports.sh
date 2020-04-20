#!/usr/bin/env bash
#
# Checks for wildcard imports (which are to be avoided at all costs). Must be
# run from root Tock directory.
#
# Author: Pat Pannuto <pat.pannuto@gmail.com>
# Author: Brad Campbell <bradjc5@gmail.com>
#
set -e

# Verify that we're running in the base directory
if [ ! -x tools/check_wildcard_imports.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

set +e
let FAIL=0
set -e

# Clippy does have an option for this -- wildcard_imports -- but we
# have not yet enabled clippy in CI so for now we check manually here

# Find folders with Cargo.toml files in them and check them (avoids matching
# this script!). Use a minimum depth of 2 to avoid matching the workspace's
# Cargo.toml (located in the top-level directory).
for f in $(find . -mindepth 2 | grep Cargo.toml); do
	pushd $(dirname $f) > /dev/null
	if $(git grep -q 'use .*\*;' -- ':!src/macros.rs'); then
		echo
		echo "$(tput bold)Wildcard import(s) found in $(dirname $f).$(tput sgr0)"
		echo "Tock style rules prohibit this use of wildcard imports."
		echo
		echo "The following wildcard imports were found:"
		git grep 'use .*\*;'
		let FAIL=FAIL+1
	fi
	popd > /dev/null
done

if [[ $FAIL -ne 0 ]]; then
	echo
	echo "$(tput bold)Formatting errors.$(tput sgr0)"
	echo "See above for details"
fi
exit $FAIL
