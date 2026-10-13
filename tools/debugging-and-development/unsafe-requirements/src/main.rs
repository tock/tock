// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! `unsafe-requirements` scans a Rust crate for `unsafe { ... }` blocks,
//! works out which specific operations inside each block are the reason it
//! has to be `unsafe` (dereferencing a raw pointer, calling an unsafe
//! function, accessing a mutable static, a union field, or inline assembly),
//! and annotates the block with `// unsafe-requirements: ...` comment lines
//! linking to the Rust documentation that explains that operation.
//!
//! This is a syntactic, best-effort tool: it does not run the Rust type
//! checker, so some operations (particularly raw pointer dereferences, which
//! look identical to a reference dereference in syntax alone) are flagged
//! more broadly than a human reviewer would, and others may be missed
//! entirely. See the README in this directory for details and limitations.
//! It is meant to help a developer quickly find documentation relevant to an
//! `unsafe` block, not to replace a human-written `// SAFETY: ...`
//! justification.

mod annotate;
mod inventory;
mod safety_docs;
mod scan;
mod urls;

use clap::Parser;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Scans a Rust crate for `unsafe` blocks and annotates each one with links
/// to the Rust documentation for the operations inside it that require
/// `unsafe`.
#[derive(Parser)]
struct Args {
    /// Root directory to scan: a crate root, a workspace root, or any
    /// directory containing Rust source files.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Insert/update the generated reference comments in the source files.
    /// Without this flag, the tool only prints a report of what it found.
    #[arg(short, long)]
    write: bool,
}

fn collect_rs_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkBuilder::new(root).build() {
        let entry = entry?;
        if entry.file_type().is_some_and(|t| t.is_file())
            && entry.path().extension().and_then(|e| e.to_str()) == Some("rs")
        {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let files = collect_rs_files(&args.path)?;

    // First pass: parse every file and build a crate-wide inventory of
    // unsafe fns/methods, mutable statics, and union fields, so that the
    // second pass can recognize uses of them regardless of which file they
    // were declared in.
    let mut parsed_files = Vec::with_capacity(files.len());
    let mut inv = inventory::Inventory::default();
    for path in files {
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("warning: could not read {}: {err}", path.display());
                continue;
            }
        };
        let parsed = match syn::parse_file(&text) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("warning: could not parse {}: {err}", path.display());
                continue;
            }
        };
        inv.visit_file(&parsed);
        parsed_files.push((path, text, parsed));
    }

    // Second pass: find unsafe blocks and classify the operations in each.
    let mut total_blocks = 0usize;
    let mut classified_blocks = 0usize;
    let mut files_changed = 0usize;

    for (path, text, parsed) in &parsed_files {
        let blocks = scan::find_unsafe_blocks(parsed, &inv);
        if blocks.is_empty() {
            continue;
        }

        total_blocks += blocks.len();
        classified_blocks += blocks.iter().filter(|b| !b.reasons.is_empty()).count();

        println!("{}", path.display());
        for block in &blocks {
            if block.reasons.is_empty() {
                println!(
                    "  line {}: unsafe block -- no automatically recognized operation (needs manual review)",
                    block.start.line
                );
            } else {
                println!("  line {}: unsafe block", block.start.line);
                for reason in &block.reasons {
                    match &reason.url {
                        Some(url) => println!("    - {}: {url}", reason.label),
                        None => println!("    - {}", reason.label),
                    }
                }
            }
        }

        if args.write
            && let Some(new_text) = annotate::annotate_file(text, &blocks)
        {
            std::fs::write(path, new_text)?;
            files_changed += 1;
        }
    }

    println!();
    println!(
        "scanned {} file(s), found {total_blocks} unsafe block(s), {classified_blocks} with a recognized operation",
        parsed_files.len()
    );
    if args.write {
        println!("updated {files_changed} file(s)");
    } else if total_blocks > 0 {
        println!("(dry run -- pass --write to insert the reference comments)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{annotate, inventory::Inventory, scan};
    use std::collections::BTreeMap;

    const FIXTURE: &str = include_str!("../testdata/basic.rs");

    fn blocks_by_line() -> BTreeMap<usize, Vec<String>> {
        let parsed = syn::parse_file(FIXTURE).expect("fixture must parse");
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        scan::find_unsafe_blocks(&parsed, &inv)
            .into_iter()
            .map(|b| {
                let labels = b.reasons.into_iter().map(|r| r.label).collect();
                (b.start.line, labels)
            })
            .collect()
    }

    #[test]
    fn recognizes_each_operation_category() {
        let blocks = blocks_by_line();

        let deref = blocks
            .values()
            .find(|l| l.len() == 1 && l[0].contains("dereferencing a raw pointer"));
        assert!(
            deref.is_some(),
            "expected a raw pointer deref block: {blocks:?}"
        );

        let call_and_static = blocks.values().find(|l| {
            l.iter().any(|s| s.contains("mutable static"))
                && l.iter().any(|s| s.contains("unsafe Tock function"))
        });
        assert!(
            call_and_static.is_some(),
            "expected a block with both a mutable static access and an unsafe call: {blocks:?}"
        );

        let transmute = blocks
            .values()
            .find(|l| l.iter().any(|s| s.contains("`transmute`")));
        assert!(
            transmute.is_some(),
            "expected a transmute call to be recognized: {blocks:?}"
        );

        let union = blocks
            .values()
            .find(|l| l.iter().any(|s| s.contains("union field access")));
        assert!(
            union.is_some(),
            "expected a union field access to be recognized: {blocks:?}"
        );

        let asm_present = blocks
            .values()
            .any(|l| l.iter().any(|s| s.contains("inline assembly")));
        assert!(
            !asm_present,
            "fixture has no inline asm, but one was reported: {blocks:?}"
        );
    }

    #[test]
    fn nested_unsafe_blocks_each_own_their_operations() {
        let blocks = blocks_by_line();
        // The outer block of `nested_example` only dereferences `ptr`; the
        // unsafe call to `danger` is inside the nested block and must not
        // also be attributed to the outer one.
        let outer = blocks.get(&40).expect("outer block at line 40");
        assert_eq!(outer.len(), 1);
        assert!(outer[0].contains("dereferencing a raw pointer"));

        let inner = blocks.get(&42).expect("inner block at line 42");
        assert_eq!(inner.len(), 1);
        assert!(inner[0].contains("unsafe Tock function `danger`"));
    }

    #[test]
    fn in_crate_unsafe_function_has_no_url() {
        let parsed = syn::parse_file(FIXTURE).unwrap();
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        let blocks = scan::find_unsafe_blocks(&parsed, &inv);

        let block = blocks
            .iter()
            .find(|b| b.reasons.iter().any(|r| r.label.contains("`danger`")))
            .expect("a block calling `danger` should exist");
        let reason = block
            .reasons
            .iter()
            .find(|r| r.label.contains("`danger`"))
            .unwrap();
        assert_eq!(reason.label, "calling unsafe Tock function `danger`");
        assert!(
            reason.url.is_none(),
            "an in-crate unsafe function should not get an external doc URL: {:?}",
            reason.url
        );
    }

    #[test]
    fn known_callable_with_safety_text_renders_structured_comment() {
        let parsed = syn::parse_file(FIXTURE).unwrap();
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        let blocks = scan::find_unsafe_blocks(&parsed, &inv);

        let new_text = annotate::annotate_file(FIXTURE, &blocks).expect("fixture should change");
        // The comment sits directly above the `ptr.add(1)` call itself;
        // find it by locating that call and walking backward to the
        // nearest preceding heading line.
        let idx = new_text
            .find("unsafe { ptr.add(1) }")
            .expect("add_example's unsafe block should exist");
        let preceding = &new_text[..idx];
        let comment_start = preceding
            .rfind("// SAFETY REQUIREMENTS:")
            .expect("heading line");
        let comment = &new_text[comment_start..idx];

        assert!(comment.contains("// SAFETY REQUIREMENTS:"));
        assert!(comment.contains("// - calling unsafe method `add`"));
        assert!(
            comment.contains(
                "//   - https://doc.rust-lang.org/core/primitive.pointer.html#method.add"
            )
        );
        assert!(comment.contains(
            "//   - If any of the following conditions are violated, the result is Undefined Behavior:"
        ));
        assert!(comment.contains("//     - The offset in bytes"));
        assert!(comment.contains("//         Allocations can never be larger"));
    }

    #[test]
    fn known_callable_without_safety_section_is_unknown() {
        let parsed = syn::parse_file(FIXTURE).unwrap();
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        let blocks = scan::find_unsafe_blocks(&parsed, &inv);

        let block = blocks
            .iter()
            .find(|b| b.reasons.iter().any(|r| r.label.contains("`transmute`")))
            .expect("a block calling `transmute` should exist");
        let reason = block
            .reasons
            .iter()
            .find(|r| r.label.contains("`transmute`"))
            .unwrap();
        assert!(reason.url.is_some());
        assert_eq!(
            reason.safety, None,
            "transmute's docs have no dedicated Safety section"
        );
    }

    #[test]
    fn write_upgrades_legacy_annotation_instead_of_duplicating() {
        let legacy = FIXTURE.replacen(
            "pub fn deref_example(ptr: *const u32) -> u32 {\n    unsafe { *ptr }\n}",
            "pub fn deref_example(ptr: *const u32) -> u32 {\n    //\n    // unsafe-requirements: dereferencing a raw pointer: https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-deref\n    unsafe { *ptr }\n}",
            1,
        );
        assert_ne!(legacy, FIXTURE, "the replacement should have applied");

        let parsed = syn::parse_file(&legacy).unwrap();
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        let blocks = scan::find_unsafe_blocks(&parsed, &inv);

        let upgraded =
            annotate::annotate_file(&legacy, &blocks).expect("legacy annotation should change");
        assert!(!upgraded.contains("unsafe-requirements:"));
        assert!(upgraded.contains("// SAFETY REQUIREMENTS:"));
        assert!(upgraded.contains("// - dereferencing a raw pointer"));

        // And it should now match a from-scratch run exactly.
        let parsed_again = syn::parse_file(&upgraded).unwrap();
        let mut inv_again = Inventory::default();
        inv_again.visit_file(&parsed_again);
        let blocks_again = scan::find_unsafe_blocks(&parsed_again, &inv_again);
        assert!(annotate::annotate_file(&upgraded, &blocks_again).is_none());
    }

    #[test]
    fn unclassified_block_is_left_unannotated() {
        let blocks = blocks_by_line();
        let boring = blocks.get(&48).expect("boring block at line 48");
        assert!(boring.is_empty());
    }

    #[test]
    fn annotate_is_idempotent() {
        let parsed = syn::parse_file(FIXTURE).unwrap();
        let mut inv = Inventory::default();
        inv.visit_file(&parsed);
        let blocks = scan::find_unsafe_blocks(&parsed, &inv);

        let once =
            annotate::annotate_file(FIXTURE, &blocks).expect("first run should change the file");
        let blocks_again = {
            let parsed = syn::parse_file(&once).unwrap();
            let mut inv = Inventory::default();
            inv.visit_file(&parsed);
            scan::find_unsafe_blocks(&parsed, &inv)
        };
        assert!(annotate::annotate_file(&once, &blocks_again).is_none());
    }
}
