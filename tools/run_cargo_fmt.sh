#!/usr/bin/env bash
#
# Runs rustfmt on all subdirectories with a Cargo.toml file. Must be run from
# root Tock directory. Protects user from inadvertently overwriting files.
#
# Author: Pat Pannuto <pat.pannuto@gmail.com>
# Author: Brad Campbell <bradjc5@gmail.com>
#
set -e

# Verify that we're running in the base directory
if [ ! -x tools/run_cargo_fmt.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

# Add the rustfmt component if needed.
if ! rustup component list | grep 'rustfmt.*(installed)' -q; then
	# Some versions of OS X want the -preview version, retry that on failure
	rustup component add rustfmt || rustup component add rustfmt-preview
fi

# Format overwrites changes, which is probably good, but it's nice to see
# what it has done
#
# `git status --porcelain` formats things for scripting
# | M changed file, unstaged
# |M  changed file, staged (git add has run)
# |MM changed file, some staged and some unstaged changes (git add then changes)
# |?? untracked file
if git status --porcelain | grep '^.M.*\.rs' -q; then
	echo "$(tput bold)Warning: Formatting will overwrite files in place.$(tput sgr0)"
	echo "While this is probably what you want, it's often useful to"
	echo "stage all of your changes (git add ...) before format runs,"
	echo "just so you can double-check everything."
	echo ""
	echo "$(tput bold)git status:$(tput sgr0)"
	git status
	echo ""
	read -p "Continue formatting with unstaged changes? [y/N] " response
	if [[ ! ( "$(echo "$response" | tr :upper: :lower:)" == "y" ) ]]; then
		exit 0
	fi
fi

set +e

if [ "$1" == "diff" ]; then
	# Just print out diffs, used by Travis.
	CARGO_FMT_ARGS="-- --check"
fi

# Format all crates in the workspace.
cargo fmt --all $CARGO_FMT_ARGS
FAIL=$?

if [[ $FAIL -ne 0 ]]; then
	echo
	echo "$(tput bold)Formatting errors.$(tput sgr0)"
	echo "See above for details"
fi
exit $FAIL
