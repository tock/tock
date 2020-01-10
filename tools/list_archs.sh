#!/bin/bash

# Find archs based on folders with Cargo.toml
for b in $(find arch -maxdepth 4 -name 'Cargo.toml'); do
    b1=${b#arch/}
    b2=${b1%/*}
    echo $b2
done
