#!/usr/bin/env bash

# Find boards based on folders with Makefiles
boards=""
for b in $(find boards | egrep 'Makefile$'); do
    b1=${b#boards/}
    b2=${b1%/*}
    boards+="$b2 "
done

for board in $boards; do
    echo $board
done
