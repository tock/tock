#!/usr/bin/env bash

# Builds all of the board documentation into doc/rustdoc.

set -e

# Delete any old docs
rm -rf doc/rustdoc

# Make the documentation for all the boards and move it to doc/rustdoc.
make alldoc
cp -r target/doc doc/rustdoc

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > doc/rustdoc/_redirects << EOF
# While we don't have a home page :/
/            /kernel            302
EOF
