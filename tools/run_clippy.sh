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
# - `clippy::manual_unwrap_or_default`: As of Apr 2024, this lint has many false
#   positives.
CLIPPY_ARGS="
-A clippy::restriction

-A clippy::if_same_then_else
-A clippy::manual_unwrap_or_default
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


-A clippy::zero_prefixed_literal
-A clippy::needless-if


-A clippy::unnecessary_unwrap
-A clippy::explicit_auto_deref
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

-A clippy::blocks_in_conditions
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
-A clippy::new-ret-no-self
-A clippy::new_without_default
-A clippy::redundant_closure
-A clippy::result_unit_err
-A clippy::single_match
-A clippy::upper_case_acronyms


-A clippy::declare-interior-mutable-const
-A clippy::from-over-into
-A clippy::let_and_return
-A clippy::missing_safety_doc
-A clippy::needless-range-loop
-A clippy::option_map_or_none
-A clippy::redundant_field_names
-A clippy::redundant_pattern_matching
-A clippy::unusual-byte-groupings
-A clippy::wrong-self-convention
-A clippy::doc_lazy_continuation
"

# Disallow all perf lints, then re-allow each one Tock does not comply with.
CLIPPY_ARGS_PERF="
-D clippy::perf

-A clippy::large-enum-variant
"

# Disallow all cargo lints, then re-allow each one Tock does not comply with.
CLIPPY_ARGS_CARGO="
-D clippy::cargo

-A clippy::cargo_common_metadata
-A clippy::negative-feature-names
"

# Disallow all nursery lints, then re-allow each one Tock does not comply with.
CLIPPY_ARGS_NURSERY="
-D clippy::nursery

-A clippy::use_self
-A clippy::option_if_let_else
-A clippy::cognitive_complexity
-A clippy::or_fun_call
-A clippy::collection_is_never_read


-A clippy::manual_clamp
-A clippy::unused_peekable
-A clippy::branches_sharing_code


-A clippy::missing_const_for_fn
-A clippy::redundant_pub_crate
-A clippy::equatable_if_let
-A clippy::fallible_impl_from
-A clippy::derive_partial_eq_without_eq
-A clippy::empty_line_after_doc_comments
-A clippy::trait_duplication_in_bounds
-A clippy::useless_let_if_seq
-A clippy::as_ptr_cast_mut
-A clippy::unnecessary_struct_initialization
-A clippy::type_repetition_in_bounds
"

# Disallow all pedantic lints, then re-allow each one Tock does not comply with.
CLIPPY_ARGS_PEDANTIC="
-D clippy::pedantic

-A clippy::doc_markdown
-A clippy::missing_errors_doc
-A clippy::if_not_else
-A clippy::cast_sign_loss
-A clippy::too_many_lines
-A clippy::must_use_candidate
-A clippy::manual_let_else
-A clippy::single_match_else
-A clippy::inline_always
-A clippy::module_name_repetitions
-A clippy::unnested-or-patterns
-A clippy::redundant_else
-A clippy::return_self_not_must_use
-A clippy::match_same_arms
-A clippy::explicit_iter_loop
-A clippy::similar_names
-A clippy::unnecessary_wraps
-A clippy::manual_assert
-A clippy::transmute_ptr_to_ptr
-A clippy::struct_excessive_bools
-A clippy::fn_params_excessive_bools
-A clippy::trivially_copy_pass_by_ref
-A clippy::borrow_as_ptr
-A clippy::tuple_array_conversions
-A clippy::verbose_bit_mask
-A clippy::large_types_passed_by_value
-A clippy::no_mangle_with_rust_abi
-A clippy::struct_field_names


-A clippy::cast_lossless
-A clippy::cast_possible_truncation
-A clippy::cast_precision_loss
-A clippy::range_plus_one
-A clippy::missing_panics_doc
-A clippy::match_wildcard_for_single_variants
-A clippy::unused_self
-A clippy::cast-possible-wrap
-A clippy::uninlined_format_args
-A clippy::unreadable_literal
-A clippy::needless_pass_by_value
-A clippy::items_after_statements
-A clippy::ref_option_ref
-A clippy::match_bool
-A clippy::redundant_closure_for_method_calls
-A clippy::no_effect_underscore_binding
-A clippy::iter_without_into_iter


-A clippy::semicolon_if_nothing_returned
-A clippy::ptr_as_ptr
-A clippy::ptr_cast_constness
-A clippy::mut_mut
-A clippy::cast_ptr_alignment
-A clippy::used_underscore_binding
-A clippy::checked_conversions
"

# Uncomment this line to automatically apply fixes to match changes to the
# disallowed lints.
# FIX="--fix --allow-dirty"

cargo clippy $FIX -- $CLIPPY_ARGS_COMPLEXITY $CLIPPY_ARGS_STYLE $CLIPPY_ARGS_PERF $CLIPPY_ARGS_CARGO $CLIPPY_ARGS_NURSERY $CLIPPY_ARGS_PEDANTIC $CLIPPY_ARGS
