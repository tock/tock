#!/usr/bin/env bash

set -e

bold=$(tput bold)
normal=$(tput sgr0)

for mkfile in `find . -maxdepth 3 -name Makefile`; do
	dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir > /dev/null
	echo ""
	echo "Building $dir"
	make -j || (echo "${bold} ⤤ Failure building $dir${normal}" ; exit 1)
	popd > /dev/null
done

echo ""
echo "${bold}All Built.${normal}"
