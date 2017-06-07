#!/bin/bash

set -e

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
