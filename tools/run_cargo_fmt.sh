#!/usr/bin/env bash

# Peg a rustfmt version while things are unstable
#
# Note: We install a local copy of rustfmt so as not to interfere with any
# other use of rustfmt on the machine
RUSTFMT_VERSION=0.7.1
export RUSTUP_TOOLCHAIN=nightly-2017-11-18

if [[ $(rustc --version) != "rustc 1.23.0-nightly (6160040d8 2017-11-18)" ]]; then
	rustup install $RUSTUP_TOOLCHAIN || (echo "Failed to install rustc. Please read doc/Getting_Started.md"; exit 1)
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

# Verify that we're running in the base directory
if [ ! -x tools/run_cargo_fmt.sh ]; then
	echo ERROR: $0 must be run from the tock repository root.
	echo ""
	exit 1
fi

# CI setup has correct rustfmt install globally already
if [ ! "$CI" == "true" ]; then
	needs_install=false
	# Check to make sure that cargo format is installed
	if [ ! -x tools/local_cargo/bin/rustfmt ]; then
		needs_install=true
	elif [ $(tools/local_cargo/bin/rustfmt --version | cut -d' ' -f1) != "$RUSTFMT_VERSION" ]; then
		needs_install=true
	fi

	if $needs_install; then
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
	let FAIL=0
	for f in $(find . | grep Cargo.toml); do
		pushd $(dirname $f) > /dev/null
		cargo-fmt -- --write-mode=overwrite || let FAIL=FAIL+1
		popd > /dev/null
	done

	if [[ $FAIL -ne 0 ]]; then
		echo
		echo "$(tput bold)Error running rustfmt.$(tput sgr0)"
		echo "See above for details"
	fi
	exit $FAIL
fi
