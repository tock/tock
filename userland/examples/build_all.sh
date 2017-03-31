#!/usr/bin/env bash

NUM_JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || 4)

set -e

bold=$(tput bold)
normal=$(tput sgr0)

function opt_rebuild {
	if [ "$CI" == "true" ]; then
		echo "${bold}Rebuilding Verbose: $1${normal}"
		make V=1
	fi
}

for mkfile in `find . -maxdepth 3 -name Makefile`; do
	dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir > /dev/null
	echo ""
	echo "Building $dir"
	make -j $NUM_JOBS || (echo "${bold} ⤤ Failure building $dir${normal}" ; opt_rebuild $dir; exit 1)
	popd > /dev/null
done

echo ""
echo "${bold}All Built.${normal}"
