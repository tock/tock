#!/usr/bin/env bash

# Install clippy if it is not already preset.
if ! rustup component list | grep 'clippy.*(installed)' -q; then
	rustup component add clippy || rustup component add clippy-preview
fi

# We start by turning most lints off (by -A with most of the categories), then
# specifically turn on lints that make sense. We do keep `clippy::correctness`
# on.
#
# There are some lints we specifically do not want:
#
# - `clippy::if_same_then_else`: There are often good reasons to enumerate
#   different states that have the same effect.

CLIPPY_ARGS="
-A clippy::complexity
-A clippy::pedantic
-A clippy::nursery
-A clippy::style
-A clippy::perf
-A clippy::cargo
-A clippy::restriction

-A clippy::if_same_then_else
-A clippy::enum_clike_unportable_variant

-D clippy::needless_return
-D clippy::unnecessary_mut_passed
-D clippy::empty_line_after_outer_attr
-D clippy::option_map_unwrap_or
-D clippy::option_map_unwrap_or_else
"

cargo clippy -- $CLIPPY_ARGS
