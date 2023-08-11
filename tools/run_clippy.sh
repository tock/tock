#!/usr/bin/env bash

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

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
CLIPPY_ARGS="
-A clippy::pedantic
-A clippy::nursery
-A clippy::style
-A clippy::perf
-A clippy::cargo
-A clippy::restriction

-A clippy::if_same_then_else

-D clippy::needless_return
-D clippy::unnecessary_mut_passed
-D clippy::empty_line_after_outer_attr
-D clippy::default_trait_access
-D clippy::map_unwrap_or
-D clippy::wildcard_imports
"

# Disallow all complexity lints, then re-allow each one Tock does not comply
# with.
#
# There are three sections:
# 1. The first section are lints we almost certainly don't want.
# 2. The section section are lints we may not want, we probably have to see the
#    resulting diff.
# 3. The third section are lints that we do want we just need to fixup the code
#    to pass the lint checks.
CLIPPY_ARGS_COMPLEXITY="
-D clippy::complexity
-A clippy::too_many_arguments
-A clippy::type_complexity
-A clippy::option_map_unit_fn
-A clippy::nonminimal_bool
-A clippy::identity-op
-A clippy::while-let-loop
-A clippy::only_used_in_recursion
-A clippy::manual-range-patterns
-A clippy::manual-flatten


-A clippy::bool_comparison
-A clippy::zero_prefixed_literal
-A clippy::needless-if


-A clippy::unnecessary_cast
-A clippy::extra_unused_lifetimes
-A clippy::unnecessary_unwrap
-A clippy::needless_lifetimes
-A clippy::useless_conversion
-A clippy::precedence
-A clippy::redundant_slicing
-A clippy::derivable_impls
-A clippy::char_lit_as_u8
-A clippy::needless_bool
-A clippy::useless_asref
-A clippy::clone-on-copy
-A clippy::explicit_auto_deref
-A clippy::explicit_counter_loop
-A clippy::manual_unwrap_or
-A clippy::borrow_deref_ref
-A clippy::overflow_check_conditional
-A clippy::needless-match
-A clippy::match-single-binding
"

# Uncomment this line to automatically apply fixes to match changes to the
# disallowed lints.
# FIX="--fix --allow-dirty"

cargo clippy $FIX -- $CLIPPY_ARGS $CLIPPY_ARGS_COMPLEXITY
