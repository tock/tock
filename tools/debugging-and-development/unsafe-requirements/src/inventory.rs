// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A crate-wide first pass that records the names of things that make
//! calling or touching them unsafe: `unsafe fn`s (free functions, inherent
//! and trait methods, and `extern` block functions), `static mut` variables,
//! and `union` fields.
//!
//! The scan pass in [`crate::scan`] uses this inventory to recognize calls
//! into unsafe functions defined elsewhere in the crate, in addition to the
//! well-known standard library functions it already knows about.

use std::collections::HashSet;
use syn::visit::Visit;
use syn::{ForeignItem, ImplItemFn, ItemFn, ItemForeignMod, ItemStatic, ItemUnion, TraitItemFn};

#[derive(Default)]
pub struct Inventory {
    /// Names of unsafe functions/methods, keyed only by their final
    /// identifier (no type or module information is available from syntax
    /// alone).
    pub unsafe_callables: HashSet<String>,
    /// Names of `static mut` variables.
    pub mutable_statics: HashSet<String>,
    /// Field names declared on any `union` in the crate.
    pub union_fields: HashSet<String>,
}

impl Inventory {
    pub fn visit_file(&mut self, file: &syn::File) {
        let mut collector = Collector { inv: self };
        collector.visit_file(file);
    }
}

struct Collector<'a> {
    inv: &'a mut Inventory,
}

impl<'a, 'ast> Visit<'ast> for Collector<'a> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if node.sig.unsafety.is_some() {
            self.inv.unsafe_callables.insert(node.sig.ident.to_string());
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if node.sig.unsafety.is_some() {
            self.inv.unsafe_callables.insert(node.sig.ident.to_string());
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if node.sig.unsafety.is_some() {
            self.inv.unsafe_callables.insert(node.sig.ident.to_string());
        }
        syn::visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast ItemForeignMod) {
        // Every function declared in an `extern` block is unsafe to call,
        // regardless of whether the block itself is written as `unsafe
        // extern`.
        for item in &node.items {
            if let ForeignItem::Fn(f) = item {
                self.inv.unsafe_callables.insert(f.sig.ident.to_string());
            }
        }
        syn::visit::visit_item_foreign_mod(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast ItemStatic) {
        if !matches!(node.mutability, syn::StaticMutability::None) {
            self.inv.mutable_statics.insert(node.ident.to_string());
        }
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_union(&mut self, node: &'ast ItemUnion) {
        for field in &node.fields.named {
            if let Some(ident) = &field.ident {
                self.inv.union_fields.insert(ident.to_string());
            }
        }
        syn::visit::visit_item_union(self, node);
    }
}
