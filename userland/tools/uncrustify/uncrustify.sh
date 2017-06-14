#!/bin/bash

set -e

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

# install uncrustify if it's missing
if ! command -v uncrustify >/dev/null; then
  echo "Formatting requires the uncrustify utility, which is not installed"
  case "$OSTYPE" in
    darwin*)
      if command -v brew; then
        echo "You have homebrew installed, press enter to automatically run"
        echo "  brew install uncrustify"
        echo "Or Ctrl-C to quit"
        read
        set -x
        brew install uncrustify
        set +x
      else
        echo "Cannot auto-install uncrustify"
        exit 1
      fi
      ;;
    linux*)
      echo "LINUX"
      if command -v apt; then
        echo "You have apt installed, press enter to automatically run"
        echo "  sudo apt install uncrustify"
        echo "Or Ctrl-C to quit"
        read
        set -x
        sudo apt install uncrustify
        set +x
      elif command -v pacman; then
        echo "You have pacman installed, press enter to automatically run"
        echo "  sudo pacman -S uncrustify"
        echo "Or Ctrl-C to quit"
        read
        set -x
        sudo packman -S uncrustify
        set +x
      else
        echo "Cannot auto-install uncrustify"
        exit 1
      fi
      ;;
    *)
      echo "unknown: $OSTYPE"
      echo "Cannot auto-install uncrustify"
      exit 1
      ;;
  esac
fi

# Validate uncrustify version
VERSION=$(uncrustify --version | egrep -o '0.[0-9]+' | cut -d '.' -f2)
if [[ "$VERSION" < 59 ]]; then
  echo ""
  echo "$(tput bold)Your uncrustify version is too old. >= v0.59 is required.$(tput sgr0)"
  echo ""
  echo "uncrustify --version"
  uncrustify --version
  echo ""
  exit 1
fi

set +e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
COMMON_FLAGS="-c $SCRIPT_DIR/uncrustify.cfg"
if [ "$CI" == "true" ]; then
  uncrustify $COMMON_FLAGS --check "$@"
  if [ $? -ne 0 ]; then
    uncrustify $COMMON_FLAGS --if-changed "$@"
    for f in $(ls *.uncrustify); do
      diff -y ${f%.*} $f
    done
    exit 1
  fi
else
  exec uncrustify $COMMON_FLAGS --no-backup "$@"
fi
