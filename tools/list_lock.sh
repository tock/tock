#!/usr/bin/env bash

# Find crates based on folders with Cargo.lock files
for b in $(find . -maxdepth 4 | egrep 'Cargo.lock$'); do
    b2=${b%/*}
    echo $b2
done
