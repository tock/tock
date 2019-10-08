#!/bin/bash

# Find chips based on folders with Cargo.toml
for b in $(find chips -maxdepth 4 -name 'Cargo.toml'); do
    b1=${b#chips/}
    b2=${b1%/*}
    echo $b2
done
