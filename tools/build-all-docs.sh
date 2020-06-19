#!/usr/bin/env bash

# Builds all of the board documentation into doc/rustdoc.

set -e

# Delete any old docs
rm -rf doc/rustdoc

# Use copy-on-write cp if available
touch _COW
if `cp -c _COW _COW2 2> /dev/null`; then
    # BSD (OS X) default
    CP_COW="cp -c"
elif `cp --reflink=auto _COW _COW2 2> /dev/null`; then
    # Coreutils (unix) default
    CP_COW="cp --reflink=auto"
else
    echo "$(tput bold)Warn: No copy-on-write cp available. Doc build will be slower.$(tput sgr0)"
    CP_COW="cp"
fi
rm -f _COW _COW2

# Make the documentation for all the boards and move it to doc/rustdoc.
make alldoc
$CP_COW -r target/doc doc/rustdoc

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > doc/rustdoc/_redirects << EOF
# While we don't have a home page :/
/            /kernel            302
EOF
