#!/usr/bin/env bash

set -e
for mkfile in `find . -maxdepth 3 -name Makefile`; do
        dir=`dirname $mkfile`
	if [ $dir == "." ]; then continue; fi
	pushd $dir
	make
	popd
done
