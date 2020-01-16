#!/usr/bin/env bash

# Find boards based on folders with Makefiles
for b in $(find boards -maxdepth 4 | egrep 'Makefile$'); do
    b1=${b#boards/}
    b2=${b1%/*}
    echo $b2
done
