#!/bin/bash

set -e
set -u

# Wrapper script that will help install uncrustify if it's missing
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

exec uncrustify "$@"
