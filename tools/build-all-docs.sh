#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

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

# Make the documentation for all the boards, for the host's native target.
cargo doc

# Replace the default rust logo with our own Tock logo and the favicon with our
# own favicon. Note, it is also possible to set this using a `#[doc]` attribute
# (https://doc.rust-lang.org/rustdoc/the-doc-attribute.html#html_logo_url) but
# doing it this way avoids having to set the attribute for every crate.
curl https://www.tockos.org/assets/img/tocklogo.png --output target/doc/rust-logo.png
curl https://www.tockos.org/assets/img/icons/favicon-32x32.png --output target/doc/favicon-32x32.png
curl https://www.tockos.org/assets/img/icons/favicon-16x16.png --output target/doc/favicon-16x16.png
curl https://www.tockos.org/assets/img/icons/safari-pinned-tab.svg --output target/doc/favicon.svg

# Move the docs to doc/rustdoc.
$CP_COW -r target/doc doc/rustdoc

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > doc/rustdoc/_redirects << EOF
# While we don't have a home page :/
/            /kernel            302
EOF
