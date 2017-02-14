#!/usr/bin/env bash

set -e

for mkfile in `find . -maxdepth 3 -name Makefile`; do
	dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir > /dev/null
	make clean > /dev/null
	popd > /dev/null
done

echo ""
echo "Done"
