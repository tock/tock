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
-D clippy::needless_borrow
"

# Disallow all complexity lints, then re-allow each one Tock does not comply
# with.
#
# There are three sections:
# 1. The first section are lints we almost certainly don't want.
# 2. The second section are lints we may not want, we probably have to see the
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

# Disallow all style lints, then re-allow each one Tock does not comply with.
#
# There are three sections:
# 1. The first section are lints we almost certainly don't want.
# 2. The second section are lints we may not want, we probably have to see the
#    resulting diff.
# 3. The third section are lints that we do want we just need to fixup the code
#    to pass the lint checks.
CLIPPY_ARGS_STYLE="
-D clippy::style

-A clippy::blocks-in-if-conditions
-A clippy::collapsible_else_if
-A clippy::collapsible_if
-A clippy::collapsible_match
-A clippy::comparison_chain
-A clippy::enum-variant-names
-A clippy::field-reassign-with-default
-A clippy::get_first
-A clippy::len_without_is_empty
-A clippy::len_zero
-A clippy::manual-map
-A clippy::manual_range_contains
-A clippy::match_like_matches_macro
-A clippy::module_inception
-A clippy::neg-multiply
-A clippy::new-ret-no-self
-A clippy::new_without_default
-A clippy::redundant_closure
-A clippy::result_unit_err
-A clippy::single_match
-A clippy::upper_case_acronyms


-A clippy::declare-interior-mutable-const
-A clippy::from-over-into
-A clippy::let_and_return
-A clippy::manual-bits
-A clippy::missing_safety_doc
-A clippy::needless-range-loop
-A clippy::needless_late_init
-A clippy::option_map_or_none
-A clippy::question-mark
-A clippy::redundant_field_names
-A clippy::redundant_pattern_matching
-A clippy::unusual-byte-groupings
-A clippy::wrong-self-convention


-A clippy::assertions-on-constants
-A clippy::assign_op_pattern
-A clippy::bool_assert_comparison
-A clippy::excessive-precision
-A clippy::init-numbered-fields
-A clippy::let-unit-value
-A clippy::manual-saturating-arithmetic
-A clippy::match-ref-pats
-A clippy::needless_borrow
-A clippy::op-ref
-A clippy::ptr-eq
-A clippy::redundant_static_lifetimes
-A clippy::single-component-path-imports
-A clippy::unnecessary_lazy_evaluations
-A clippy::unused_unit
-A clippy::write-with-newline
-A clippy::zero_ptr
"

# Uncomment this line to automatically apply fixes to match changes to the
# disallowed lints.
# FIX="--fix --allow-dirty"

cargo clippy $FIX -- $CLIPPY_ARGS $CLIPPY_ARGS_COMPLEXITY $CLIPPY_ARGS_STYLE
