#!/usr/bin/env bash

# Find tools built in rust based on folders with Cargo.toml
for b in $(find tools -maxdepth 4 -name 'Cargo.toml'); do
    b1=${b#tools/}
    b2=${b1%/*}
    echo $b2
done
