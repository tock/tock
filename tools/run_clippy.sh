#!/usr/bin/env bash

# Install clippy if it is not already preset.
if ! rustup component list | grep 'clippy.*(installed)' -q; then
	rustup component add clippy || rustup component add clippy-preview
fi

# Notably, this runs clippy on the workspace from which it is called. When invoked
# from the root folder, as is done in CI or by invoking `make ci-job-clippy`,
# this code is not run on the rust code in tools/, as that code is in a
# separate cargo workspace.

# We start by turning most lints off (by -A with most of the categories), then
# specifically turn on lints that make sense. We do keep `clippy::correctness`
# on.
#
# There are some lints we specifically do not want:
#
# - `clippy::if_same_then_else`: There are often good reasons to enumerate
#   different states that have the same effect.
# - `clippy::borrow_interior_mutable_const`: There's a common pattern of using
#   a const `StaticRef` to reference mutable memory-mapped registers, and that
#   triggers a false positive of this lint.
#
#   See https://github.com/rust-lang/rust-clippy/issues/5796.

CLIPPY_ARGS="
-A clippy::complexity
-A clippy::pedantic
-A clippy::nursery
-A clippy::style
-A clippy::perf
-A clippy::cargo
-A clippy::restriction

-A clippy::if_same_then_else
-A clippy::borrow_interior_mutable_const

-D clippy::needless_return
-D clippy::unnecessary_mut_passed
-D clippy::empty_line_after_outer_attr
-D clippy::default_trait_access
-D clippy::map_unwrap_or
-D clippy::wildcard_imports
"

cargo clippy -- $CLIPPY_ARGS
