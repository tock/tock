#!/usr/bin/env bash

set -e

bold=$(tput bold)
normal=$(tput sgr0)

function opt_rebuild {
	if [ "$CI" == "true" ]; then
		echo "${bold}Rebuilding Verbose: $1${normal}"
		make format V=1
	fi
}

for mkfile in `find . -maxdepth 3 -name Makefile`; do
	dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	# Skip directories with leading _'s, useful for leaving test apps around
	if [[ $(basename $dir) == _* ]]; then continue; fi

	pushd $dir > /dev/null
	echo ""
	echo "Building $dir"
	make format || (echo "${bold} ⤤ Failure formatting $dir${normal}" ; opt_rebuild $dir; exit 1)
	popd > /dev/null
done

echo ""
echo "${bold}All Built.${normal}"
