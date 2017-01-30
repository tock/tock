#!/usr/bin/env bash

set -e

echo "Make clean all apps to ensure warnings are printed..."
for mkfile in `find . -maxdepth 3 -name Makefile`; do
        dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir > /dev/null
	make clean > /dev/null
	popd > /dev/null
done
echo "done"

for mkfile in `find . -maxdepth 3 -name Makefile`; do
        dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir > /dev/null
	echo ""
	echo "Building $dir"
	make
	popd > /dev/null
done
