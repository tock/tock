#!/usr/bin/env bash

set -e

bold=$(tput bold)
normal=$(tput sgr0)

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

export TOCK_NO_CHECK_UNSTAGED=1

function opt_rebuild {
	if [ "$CI" == "true" ]; then
		echo "${bold}Rebuilding Verbose: $1${normal}"
		make format V=1
	fi
}

echo ""
echo "${bold}Formatting examples${normal}"

for mkfile in `find . -maxdepth 3 -name Makefile`; do
	dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	# Skip directories with leading _'s, useful for leaving test apps around
	if [[ $(basename $dir) == _* ]]; then continue; fi

	pushd $dir > /dev/null
	echo ""
	echo "Fromatting $dir"
	make format || (echo "${bold} ⤤ Failure formatting $dir${normal}" ; opt_rebuild $dir; exit 1)
	popd > /dev/null
done

echo ""
echo "${bold}All formatted.${normal}"
