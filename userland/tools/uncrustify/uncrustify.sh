#!/bin/bash

set -e
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Format overwrites changes, which is probably good, but it's nice to see
# what it has done
#
# `git status --porcelain` formats things for scripting
# | M changed file, unstaged
# |M  changed file, staged (git add has run)
# |MM changed file, some staged and some unstaged changes (git add then changes)
# |?? untracked file
if git status --porcelain | grep '^.M.*\.[ch].*' -q; then
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

# The version we are currently using
UNCRUSTIFY_VERSION=65

if [ -x $SCRIPT_DIR/uncrustify-uncrustify-0.$UNCRUSTIFY_VERSION/build/uncrustify ]; then
  PATH="$SCRIPT_DIR/uncrustify-uncrustify-0.$UNCRUSTIFY_VERSION/build:$PATH"
fi

# Check if the right version is already installed
do_install=false
if ! command -v uncrustify >/dev/null; then
  do_install=true
else
  # Validate uncrustify version
  VERSION=$(uncrustify --version | egrep -o '0.[0-9]+' | cut -d '.' -f2)
  if [[ "$VERSION" != $UNCRUSTIFY_VERSION ]]; then
    do_install=true
  fi
fi

# install uncrustify if it's missing
if $do_install; then
  echo "$(tput bold)"
  echo "INFO: uncrustify version 0.$UNCRUSTIFY_VERSION not installed. Installing."
  echo "$(tput sgr0)(This will take a moment)"
  echo ""

  # Check that we can, cmake is probably the only thing missing
  if ! command -v cmake >/dev/null; then
    echo "$(tput bold) ERR: cmake not installed, required to build uncrustify$(tput sgr0)"
    echo ""
    echo "Please install either uncrustify version 0.$UNCRUSTIFY_VERSION or cmake"
    exit 1
  fi

  pushd "$SCRIPT_DIR" > /dev/null

  echo " * Downloading sources..."
  wget -q https://github.com/uncrustify/uncrustify/archive/uncrustify-0.$UNCRUSTIFY_VERSION.tar.gz
  tar -xzf uncrustify-0.$UNCRUSTIFY_VERSION.tar.gz
  mkdir "uncrustify-uncrustify-0.$UNCRUSTIFY_VERSION/build"
  pushd "uncrustify-uncrustify-0.$UNCRUSTIFY_VERSION/build" > /dev/null

  echo " * Building..."
  cmake .. > /dev/null
  cmake --build . > /dev/null

  echo " * Done"
  popd > /dev/null
  popd > /dev/null

  PATH="$SCRIPT_DIR/uncrustify-uncrustify-0.$UNCRUSTIFY_VERSION/build:$PATH"
  echo ""
fi

set +e

COMMON_FLAGS="-c $SCRIPT_DIR/uncrustify.cfg"
exec uncrustify $COMMON_FLAGS "$@"
