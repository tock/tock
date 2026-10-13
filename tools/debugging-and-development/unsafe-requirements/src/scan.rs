// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Finds every `unsafe { ... }` block in a parsed file and, for each one,
//! determines which specific operations inside it are the reason the block
//! has to be `unsafe`.

use crate::inventory::Inventory;
use crate::safety_docs;
use crate::urls;
use proc_macro2::LineColumn;
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprUnsafe};

/// One `unsafe { ... }` block found in a source file.
pub struct UnsafeBlock {
    /// The line/column of the `unsafe` keyword itself.
    pub start: LineColumn,
    /// The reasons this block needs to be unsafe, deduplicated and sorted.
    pub reasons: BTreeSet<Reason>,
}

/// One recognized reason an `unsafe` block needs to be unsafe.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Reason {
    /// A brief, human-readable description of the operation, e.g.
    /// `` calling unsafe method `add` ``.
    pub label: String,
    /// A documentation URL for the operation, when one applies. A call into
    /// an unsafe function/method defined in the scanned source itself has
    /// no URL: there's no external documentation to point at.
    pub url: Option<String>,
    /// The verbatim "Safety" section of the documentation at `url`, when
    /// `url` is set and that section exists. `None` here (with `url` set)
    /// means "unknown", not "this operation has no safety requirements" --
    /// not every unsafe function's documentation has a dedicated section to
    /// copy.
    pub safety: Option<&'static str>,
}

/// Finds all `unsafe` blocks in `file`, using `inv` to recognize calls into
/// unsafe functions/methods defined elsewhere in the crate.
pub fn find_unsafe_blocks(file: &syn::File, inv: &Inventory) -> Vec<UnsafeBlock> {
    let mut finder = Finder {
        inv,
        blocks: Vec::new(),
    };
    finder.visit_file(file);
    finder.blocks
}

struct Finder<'a> {
    inv: &'a Inventory,
    blocks: Vec<UnsafeBlock>,
}

impl<'a, 'ast> Visit<'ast> for Finder<'a> {
    fn visit_expr_unsafe(&mut self, node: &'ast ExprUnsafe) {
        let start = node.unsafe_token.span().start();

        let mut op_scanner = OpScanner {
            inv: self.inv,
            reasons: BTreeSet::new(),
        };
        op_scanner.visit_block(&node.block);

        self.blocks.push(UnsafeBlock {
            start,
            reasons: op_scanner.reasons,
        });

        // Keep descending so that unsafe blocks nested inside this one are
        // also recorded as their own, separate blocks. `OpScanner` (below)
        // is the one that stops at nested unsafe blocks, so operations
        // don't get attributed to both the inner and the outer block.
        syn::visit::visit_expr_unsafe(self, node);
    }
}

/// Walks the contents of a single unsafe block (and nothing past a nested
/// unsafe block, which owns its own operations) collecting the specific
/// operations that require `unsafe`.
struct OpScanner<'a> {
    inv: &'a Inventory,
    reasons: BTreeSet<Reason>,
}

impl<'a> OpScanner<'a> {
    fn note(&mut self, label: String, url: &str) {
        self.reasons.insert(Reason {
            label,
            url: Some(url.to_string()),
            safety: None,
        });
    }

    /// Like [`Self::note`], but also records the verbatim "Safety" section
    /// text for `url`, when one was found for this operation.
    fn note_with_safety(&mut self, label: String, url: &str, safety: Option<&'static str>) {
        self.reasons.insert(Reason {
            label,
            url: Some(url.to_string()),
            safety,
        });
    }

    /// Like [`Self::note`], but for a reason with no documentation URL to
    /// point at (e.g. calling an unsafe function defined in the scanned
    /// source itself).
    fn note_no_url(&mut self, label: String) {
        self.reasons.insert(Reason {
            label,
            url: None,
            safety: None,
        });
    }
}

impl<'a, 'ast> Visit<'ast> for OpScanner<'a> {
    fn visit_expr_unsafe(&mut self, _node: &'ast ExprUnsafe) {
        // Do not recurse: a nested `unsafe` block is responsible for its own
        // operations, not this (outer) one.
    }

    fn visit_expr_unary(&mut self, node: &'ast syn::ExprUnary) {
        if matches!(node.op, syn::UnOp::Deref(_)) {
            self.note("dereferencing a raw pointer".to_string(), urls::DEREF);
        }
        syn::visit::visit_expr_unary(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(p) = node.func.as_ref()
            && let Some(seg) = p.path.segments.last()
        {
            let name = seg.ident.to_string();
            if self.inv.unsafe_callables.contains(&name) {
                self.note_no_url(format!("calling unsafe Tock function `{name}`"));
            } else if urls::is_known_unsafe_callable(&name) {
                self.note_known_callable(&name, "calling unsafe function");
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let name = node.method.to_string();
        if self.inv.unsafe_callables.contains(&name) {
            self.note_no_url(format!("calling unsafe Tock method `{name}`"));
        } else if urls::is_known_unsafe_callable(&name) {
            self.note_known_callable(&name, "calling unsafe method");
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(ident) = node.path.get_ident() {
            let name = ident.to_string();
            if self.inv.mutable_statics.contains(&name) {
                self.note(format!("accessing mutable static `{name}`"), urls::STATIC);
            }
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(ident) = &node.member {
            let name = ident.to_string();
            // This is a weak heuristic: without type information we can only
            // tell that *some* union in the crate has a field with this
            // name, not that this particular field access is through that
            // union.
            if self.inv.union_fields.contains(&name) {
                self.note(
                    format!("possible union field access `.{name}`"),
                    urls::UNION,
                );
            }
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Some(seg) = node.mac.path.segments.last() {
            let name = seg.ident.to_string();
            if name == "asm" || name == "naked_asm" {
                self.note("inline assembly".to_string(), urls::ASM);
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }
}

impl<'a> OpScanner<'a> {
    /// Notes a call into a well-known `core`/`alloc` unsafe function or
    /// method named `name`, looking up both its documentation URL and (if
    /// available) its copied "Safety" section text.
    fn note_known_callable(&mut self, name: &str, verb: &str) {
        match urls::specific_url_for_callee(name) {
            Some(url) => {
                let safety = safety_docs::text_for(name);
                self.note_with_safety(format!("{verb} `{name}`"), url, safety);
            }
            None => {
                self.note(format!("{verb} `{name}`"), urls::CALL);
            }
        }
    }
}
