// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Turns the `unsafe` blocks found by [`crate::scan`] into edits of the
//! original source text: a `// unsafe-requirements: ...` comment line per
//! recognized reason, inserted immediately above the block.
//!
//! Editing is done on the original source text, by line number, rather than
//! by re-printing the parsed `syn::File` with `quote!`, so that a run of this
//! tool never changes anything else about the file (formatting, comments,
//! blank lines).

use crate::scan::UnsafeBlock;

/// Prefix used for generated comment lines, so that a later run can find and
/// replace its own previous output instead of piling up duplicates.
const MARKER: &str = "// unsafe-requirements:";

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

        let mut remove_above = 0;
        while line_idx > remove_above {
            let candidate = original_lines[line_idx - remove_above - 1];
            if candidate.starts_with(&indent) && candidate[indent.len()..].starts_with(MARKER) {
                remove_above += 1;
            } else {
                break;
            }
        }
        // The blank `//` separator line directly above a run of marker
        // lines belongs to this tool's output too, so it's removed (and
        // regenerated) along with them.
        if remove_above > 0 && line_idx > remove_above {
            let separator_candidate = original_lines[line_idx - remove_above - 1];
            if separator_candidate == format!("{indent}//") {
                remove_above += 1;
            }
        }

        let new_lines: Vec<String> = if block.reasons.is_empty() {
            Vec::new()
        } else {
            std::iter::once(format!("{indent}//"))
                .chain(block.reasons.iter().map(|(label, url)| match url {
                    Some(url) => format!("{indent}{MARKER} {label}: {url}"),
                    None => format!("{indent}{MARKER} {label}"),
                }))
                .collect()
        };

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
