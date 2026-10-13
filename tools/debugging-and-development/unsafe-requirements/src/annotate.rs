// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Turns the `unsafe` blocks found by [`crate::scan`] into edits of the
//! original source text: a `SAFETY REQUIREMENTS:` comment block, inserted
//! immediately above the block, listing each recognized reason as a
//! Markdown-style list item.
//!
//! The generated comment for one block looks like:
//!
//! ```text
//! // SAFETY REQUIREMENTS:
//! // - calling unsafe method `add`
//! //   - https://doc.rust-lang.org/core/primitive.pointer.html#method.add
//! //   - If any of the following conditions are violated, the result is Undefined Behavior:
//! //
//! //     - The offset in bytes, ..., must fit in an isize.
//! //     - ...
//! ```
//!
//! The top-level list has one item per reason; each item's own sub-list
//! has the reason's documentation URL first, then the verbatim text of
//! that page's "Safety" section (or the word `unknown` when there isn't
//! one to copy). A reason with no URL at all (a call into an unsafe
//! function/method defined in the scanned source itself) has no sub-list.
//!
//! Editing is done on the original source text, by line number, rather than
//! by re-printing the parsed `syn::File` with `quote!`, so that a run of this
//! tool never changes anything else about the file (formatting, comments,
//! blank lines).

use crate::scan::{Reason, UnsafeBlock};

/// The line that starts a generated block, used both to write it and to
/// find a previous run's output so it can be replaced rather than
/// duplicated.
const HEADING: &str = "// SAFETY REQUIREMENTS:";

/// Per-line marker used by a previous version of this tool (one
/// `// unsafe-requirements: <label>: <url>` line per reason, no heading).
/// Recognized here only so that re-running `--write` upgrades old
/// annotations to the current format instead of leaving them as orphaned
/// comments alongside a new block.
const LEGACY_LINE_MARKER: &str = "// unsafe-requirements:";

/// Renders one reason as a top-level list item (`- {label}`) followed, when
/// `reason.url` is set, by a two-item sub-list: the URL, then the copied
/// "Safety" text (or `unknown`).
fn render_reason(indent: &str, reason: &Reason, out: &mut Vec<String>) {
    out.push(format!("{indent}// - {}", reason.label));
    let Some(url) = &reason.url else {
        return;
    };
    out.push(format!("{indent}//   - {url}"));
    match reason.safety {
        Some(text) => {
            // The text's own first line is the section's opening sentence
            // (stripped of a leading `- `, in case the section starts with
            // a list rather than a lead-in paragraph); it becomes this
            // sub-item's own text. Every following line -- including blank
            // ones, which mark paragraph breaks -- is copied as-is, each
            // hanging-indented 4 columns further to align under that text.
            let mut lines = text.lines();
            let first = lines.next().unwrap_or("");
            let first = first.strip_prefix("- ").unwrap_or(first);
            out.push(format!("{indent}//   - {first}"));
            for line in lines {
                if line.is_empty() {
                    out.push(format!("{indent}//"));
                } else {
                    out.push(format!("{indent}//     {line}"));
                }
            }
        }
        None => out.push(format!("{indent}//   - unknown")),
    }
}

/// Computes the new contents of a file after inserting/updating the
/// generated comments for `blocks`. Returns `None` if nothing changes.
pub fn annotate_file(source: &str, blocks: &[UnsafeBlock]) -> Option<String> {
    let original_lines: Vec<&str> = source.lines().collect();

    struct Edit {
        /// 0-indexed line that the `unsafe` block starts on.
        at_line: usize,
        /// How many previously generated lines directly above `at_line` to
        /// remove before inserting `new_lines`.
        remove_above: usize,
        new_lines: Vec<String>,
    }

    let mut edits = Vec::new();

    for block in blocks {
        let line_idx = block.start.line.saturating_sub(1);
        if line_idx >= original_lines.len() {
            continue;
        }
        let indent: String = original_lines[line_idx]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();

        // Scan upward through the contiguous run of comment lines directly
        // above the block, looking for a previous run's `HEADING` line.
        // Everything from that line down to (but not including) `at_line`
        // is this tool's own output and gets replaced; anything above it
        // (a human-written `// SAFETY: ...` comment, say) is left alone.
        let heading_line = format!("{indent}{HEADING}");
        let blank_line = format!("{indent}//");
        let mut probe = line_idx;
        let mut marker_at = None;
        while probe > 0 {
            let candidate = original_lines[probe - 1];
            if !candidate.starts_with(&indent) || !candidate.trim_start().starts_with("//") {
                break;
            }
            probe -= 1;
            if candidate == heading_line {
                marker_at = Some(probe);
                break;
            }
        }
        let remove_above = match marker_at {
            Some(mut at) => {
                // Absorb the blank `//` separator line directly above the
                // heading too, since this tool always writes one.
                if at > 0 && original_lines[at - 1] == blank_line {
                    at -= 1;
                }
                line_idx - at
            }
            None => {
                // No current-format heading; check for a contiguous run of
                // `LEGACY_LINE_MARKER` lines directly above the block
                // instead, left by a previous version of this tool.
                let legacy_prefix = format!("{indent}{LEGACY_LINE_MARKER}");
                let mut at = line_idx;
                while at > 0 && original_lines[at - 1].starts_with(&legacy_prefix) {
                    at -= 1;
                }
                if at < line_idx {
                    if at > 0 && original_lines[at - 1] == blank_line {
                        at -= 1;
                    }
                    line_idx - at
                } else {
                    0
                }
            }
        };

        let mut new_lines: Vec<String> = Vec::new();
        if !block.reasons.is_empty() {
            new_lines.push(format!("{indent}//"));
            new_lines.push(heading_line);
            for reason in &block.reasons {
                render_reason(&indent, reason, &mut new_lines);
            }
        }

        if remove_above == 0 && new_lines.is_empty() {
            continue;
        }

        edits.push(Edit {
            at_line: line_idx,
            remove_above,
            new_lines,
        });
    }

    if edits.is_empty() {
        return None;
    }

    edits.sort_by_key(|e| e.at_line);

    let mut lines: Vec<String> = original_lines.iter().map(|s| s.to_string()).collect();
    for edit in edits.into_iter().rev() {
        let remove_start = edit.at_line - edit.remove_above;
        lines.splice(remove_start..edit.at_line, edit.new_lines);
    }

    let mut result = lines.join("\n");
    if source.ends_with('\n') {
        result.push('\n');
    }

    if result == source { None } else { Some(result) }
}
