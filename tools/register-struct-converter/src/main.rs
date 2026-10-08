// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Converts old-style `#[repr(C)]` register structs into the
//! `register_structs! { ... }` macro format used throughout Tock's chip
//! crates.
//!
//! Given a Rust source file, this tool finds every `#[repr(C)]` struct whose
//! fields are register wrapper types (`ReadWrite<...>`, `ReadOnly<...>`,
//! `WriteOnly<...>`, `Aliased<...>`, ...), possibly in arrays, and possibly
//! interspersed with manual padding fields (`_reserved: [u8; N]`). It
//! computes the byte offset of every field from the sizes of the preceding
//! fields and rewrites the struct using the `register_structs!` macro, which
//! encodes those offsets explicitly and statically checks them.
//!
//! Usage:
//!
//! ```text
//! register-struct-converter <path/to/file.rs>
//! ```
//!
//! The file is rewritten in place. Run `cargo fmt` on the result afterwards;
//! this tool does not attempt to perfectly match rustfmt's output.
//!
//! Limitations: array lengths and padding sizes must be literal integers (or
//! simple +, -, *, / arithmetic over literals) -- named constants are not
//! evaluated. Fields whose type is not a (possibly array of) register
//! wrapper type or a raw `[u8; N]` padding array are reported as errors
//! rather than silently mishandled.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: register-struct-converter <path-to-rust-file>");
        return ExitCode::FAILURE;
    }
    let path = &args[1];

    let input = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            return ExitCode::FAILURE;
        }
    };

    match convert_file(&input) {
        Ok(Conversion {
            output,
            structs_converted,
        }) => {
            if structs_converted == 0 {
                eprintln!("no #[repr(C)] register struct found in {}", path);
                return ExitCode::FAILURE;
            }
            if let Err(e) = fs::write(path, output) {
                eprintln!("error writing {}: {}", path, e);
                return ExitCode::FAILURE;
            }
            eprintln!(
                "converted {} struct(s) in {} -- run `cargo fmt` next",
                structs_converted, path
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error converting {}: {}", path, e);
            ExitCode::FAILURE
        }
    }
}

struct Conversion {
    output: String,
    structs_converted: usize,
}

/// Finds and converts every eligible `#[repr(C)]` struct in `input`.
fn convert_file(input: &str) -> Result<Conversion, String> {
    let mut text = input.to_string();
    let mut converted = 0usize;
    let mut search_from = 0usize;

    loop {
        let found = match find_next_struct(&text, search_from) {
            None => break,
            Some(f) => f,
        };

        match convert_struct(&found) {
            Ok(replacement) => {
                let new_text = format!(
                    "{}{}{}",
                    &text[..found.block_start],
                    replacement,
                    &text[found.block_end..]
                );
                search_from = found.block_start + replacement.len();
                text = new_text;
                converted += 1;
            }
            Err(e) => {
                return Err(format!("in struct `{}`: {}", found.name, e));
            }
        }
    }

    let text = if converted > 0 {
        add_register_structs_import(&text)
    } else {
        text
    };

    Ok(Conversion {
        output: text,
        structs_converted: converted,
    })
}

/// A located `#[repr(C)] struct Name { ... }` item.
struct FoundStruct {
    /// Byte offset where replacement should start (the first doc-comment
    /// line belonging to this item, or `#[repr(C)]` if there is none).
    block_start: usize,
    /// Byte offset where replacement should end (just after the closing
    /// brace of the struct body).
    block_end: usize,
    name: String,
    vis: String,
    doc_lines: Vec<String>,
    body: String,
}

/// Scans `text` starting at byte offset `from` for the next `#[repr(C)]`
/// struct item, returning its parsed pieces.
fn find_next_struct(text: &str, from: usize) -> Option<FoundStruct> {
    let attr_pos = text[from..].find("#[repr(C)]")? + from;

    // Walk backwards from `#[repr(C)]` over any contiguous comment lines to
    // find the start of the block we will replace.
    let mut block_start = attr_pos;
    {
        let mut line_start = line_start_at(text, attr_pos);
        loop {
            if line_start == 0 {
                block_start = line_start;
                break;
            }
            let prev_line_start = line_start_at(text, line_start - 1);
            let prev_line = text[prev_line_start..line_start].trim_end_matches('\n');
            if prev_line.trim().starts_with("//") {
                block_start = prev_line_start;
                line_start = prev_line_start;
            } else {
                break;
            }
        }
    }

    // After `#[repr(C)]`, expect (only whitespace and) an optional `pub`
    // visibility, then `struct Name {`.
    let after_attr = skip_to_non_ws(text, attr_pos + "#[repr(C)]".len());
    let rest = &text[after_attr..];
    let struct_kw_pos = match rest.find("struct ") {
        Some(p) => p,
        None => return find_next_struct(text, attr_pos + 1),
    };
    let vis = rest[..struct_kw_pos].trim().to_string();
    if !vis.is_empty() && !vis.starts_with("pub") {
        // Something else (e.g. another attribute) sits between #[repr(C)]
        // and `struct`; not a shape we handle, skip this occurrence.
        return find_next_struct(text, attr_pos + 1);
    }

    let after_struct_kw = after_attr + struct_kw_pos + "struct ".len();
    let name_start = skip_to_non_ws(text, after_struct_kw);
    let name_end = {
        let i = text[name_start..].find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
        name_start + i
    };
    let name = text[name_start..name_end].to_string();

    let brace_open = {
        let i = text[name_end..].find('{')?;
        name_end + i
    };
    let between = text[name_end..brace_open].trim();
    if !between.is_empty() {
        // Generics / where-clause: not expected in register structs.
        return find_next_struct(text, attr_pos + 1);
    }

    let body_start = brace_open + 1;
    let body_end = find_matching_brace(text, brace_open)?;
    let body = text[body_start..body_end].to_string();

    let doc_lines = text[block_start..attr_pos]
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Some(FoundStruct {
        block_start,
        block_end: body_end + 1,
        name,
        vis,
        doc_lines,
        body,
    })
}

fn line_start_at(text: &str, pos: usize) -> usize {
    text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

fn skip_to_non_ws(text: &str, from: usize) -> usize {
    text[from..]
        .find(|c: char| !c.is_whitespace())
        .map(|i| from + i)
        .unwrap_or(text.len())
}

/// Given the byte offset of an opening `{`, finds the offset of its matching
/// `}`.
fn find_matching_brace(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// One field (or padding entry) parsed out of the old struct body.
struct Field {
    doc_lines: Vec<String>,
    vis: String,
    name: String,
    ty: String,
}

struct RenderedField {
    doc_lines: Vec<String>,
    vis: String,
    name: String,
    ty: String,
    size_bytes: u64,
    is_padding: bool,
}

fn convert_struct(found: &FoundStruct) -> Result<String, String> {
    let fields = split_fields(&found.body)?;

    // First pass: compute sizes and the final struct size, so every offset
    // in the struct can be printed with a consistent hex width.
    let mut rendered = Vec::with_capacity(fields.len());
    let mut used_names: HashSet<String> = HashSet::new();
    let mut total: u64 = 0;
    for field in &fields {
        let parsed = parse_type(&field.ty)
            .map_err(|e| format!("field `{}` has type `{}`: {}", field.name, field.ty, e))?;
        let name = unique_name(&field.name, &mut used_names);
        total += parsed.size_bytes;
        rendered.push(RenderedField {
            doc_lines: field.doc_lines.clone(),
            vis: field.vis.clone(),
            name,
            ty: field.ty.trim().to_string(),
            size_bytes: parsed.size_bytes,
            is_padding: parsed.is_padding,
        });
    }
    let hex_width = format!("{:X}", total).len().max(3);

    // Second pass: emit.
    let mut out = String::new();
    out.push_str("register_structs! {\n");
    for d in &found.doc_lines {
        out.push_str(&format!("    ///{}\n", prefix_space(&strip_comment_marker(d))));
    }
    let vis_prefix = if found.vis.is_empty() {
        String::new()
    } else {
        format!("{} ", found.vis)
    };
    out.push_str(&format!("    {}{} {{\n", vis_prefix, found.name));

    let mut offset: u64 = 0;
    for field in &rendered {
        for d in &field.doc_lines {
            out.push_str(&format!(
                "        ///{}\n",
                prefix_space(&strip_comment_marker(d))
            ));
        }

        let offset_str = format_offset(offset, hex_width);
        if field.is_padding {
            out.push_str(&format!("        ({} => {}),\n", offset_str, field.name));
        } else {
            let vis_prefix = if field.vis.is_empty() {
                String::new()
            } else {
                format!("{} ", field.vis)
            };
            out.push_str(&format!(
                "        ({} => {}{}: {}),\n",
                offset_str, vis_prefix, field.name, field.ty
            ));
        }

        offset += field.size_bytes;
    }

    out.push_str(&format!(
        "        ({} => @END),\n",
        format_offset(offset, hex_width)
    ));
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

fn prefix_space(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        format!(" {}", s)
    }
}

/// Strips a leading `//`, `///`, or `//!` comment marker (and one following
/// space) from a line.
fn strip_comment_marker(line: &str) -> String {
    let l = line.trim();
    let l = l
        .strip_prefix("///")
        .or_else(|| l.strip_prefix("//!"))
        .unwrap_or(l);
    let l = if l.starts_with("//") {
        l.trim_start_matches('/')
    } else {
        l
    };
    l.strip_prefix(' ').unwrap_or(l).to_string()
}

fn unique_name(name: &str, used: &mut HashSet<String>) -> String {
    if used.insert(name.to_string()) {
        return name.to_string();
    }
    let mut n = 0;
    loop {
        let candidate = format!("{}{}", name, n);
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

fn format_offset(offset: u64, hex_width: usize) -> String {
    // Lowercase hex, matching the prevailing style in existing
    // register_structs! definitions.
    format!("{:#0width$x}", offset, width = hex_width + 2)
}

/// Splits a leading `pub` / `pub(...)` visibility off a field name.
fn split_field_vis(s: &str) -> (String, String) {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("pub(")
        && let Some(close) = rest.find(')') {
            let vis = format!("pub({}", &rest[..=close]);
            return (vis, rest[close + 1..].trim().to_string());
        }
    if let Some(rest) = s.strip_prefix("pub ") {
        return ("pub".to_string(), rest.trim().to_string());
    }
    (String::new(), s.to_string())
}

/// Splits a struct body into its fields (each with any doc comments that
/// directly preceded it). Field declarations may span multiple lines; they
/// are reassembled by tracking bracket/paren/angle-bracket depth.
fn split_fields(body: &str) -> Result<Vec<Field>, String> {
    let mut fields = Vec::new();
    let mut pending_doc: Vec<String> = Vec::new();

    let mut lines = body.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") {
            pending_doc.push(trimmed.to_string());
            continue;
        }
        if trimmed.starts_with('#') {
            // Skip other attributes (e.g. #[cfg(..)]) verbatim; not
            // expected on register fields but don't choke on them.
            continue;
        }

        let mut decl = trimmed.to_string();
        while !is_complete_field_decl(&decl) {
            match lines.next() {
                Some(next) => {
                    decl.push(' ');
                    decl.push_str(next.trim());
                }
                None => return Err(format!("unterminated field declaration: `{}`", decl)),
            }
        }

        let decl = decl.trim_end_matches(',').trim().to_string();
        let colon = find_top_level_char(&decl, ':')
            .ok_or_else(|| format!("expected `name: Type` in `{}`", decl))?;
        let raw_name = decl[..colon].trim();
        let ty = decl[colon + 1..].trim().to_string();
        let (vis, name) = split_field_vis(raw_name);

        fields.push(Field {
            doc_lines: std::mem::take(&mut pending_doc),
            vis,
            name,
            ty,
        });
    }

    Ok(fields)
}

/// A declaration is "complete" once every `<>`, `[]`, `()` opened in it is
/// closed and it ends with a top-level trailing comma.
fn is_complete_field_decl(decl: &str) -> bool {
    let mut depth = 0i32;
    for c in decl.chars() {
        match c {
            '<' | '[' | '(' => depth += 1,
            '>' | ']' | ')' => depth -= 1,
            _ => {}
        }
    }
    depth == 0 && decl.trim_end().ends_with(',')
}

/// Finds the first occurrence of `target` that is not nested inside
/// `<>`/`[]`/`()`.
fn find_top_level_char(s: &str, target: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '[' | '(' => depth += 1,
            '>' | ']' | ')' => depth -= 1,
            c if c == target && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

struct ParsedType {
    size_bytes: u64,
    is_padding: bool,
}

/// Computes the size (in bytes) of a field's type, and whether it is a raw
/// `[u8; N]` padding array.
fn parse_type(ty: &str) -> Result<ParsedType, String> {
    let ty = ty.trim();
    if let Some(inner) = strip_array_brackets(ty) {
        let semi = find_top_level_char(inner, ';')
            .ok_or_else(|| format!("expected `[Type; N]`, got `{}`", ty))?;
        let elem_ty = inner[..semi].trim();
        let count_expr = inner[semi + 1..].trim();
        let count = eval_int_expr(count_expr)
            .map_err(|e| format!("couldn't evaluate array length `{}`: {}", count_expr, e))?;
        let is_padding = elem_ty == "u8";
        let elem_size = if is_padding { 1 } else { scalar_size(elem_ty)? };
        Ok(ParsedType {
            size_bytes: elem_size * count,
            is_padding,
        })
    } else {
        Ok(ParsedType {
            size_bytes: scalar_size(ty)?,
            is_padding: false,
        })
    }
}

/// If `ty` is `[ ... ]`, returns the inner text.
fn strip_array_brackets(ty: &str) -> Option<&str> {
    let ty = ty.trim();
    if ty.starts_with('[') && ty.ends_with(']') {
        Some(ty[1..ty.len() - 1].trim())
    } else {
        None
    }
}

/// Computes the size of a non-array field type: either a bare integer
/// primitive (`u8`/`u16`/`u32`/`u64`), or a register wrapper type
/// (`ReadWrite<u32>`, `ReadOnly<u16, FOO::Register>`, `Aliased<u32, ...>`,
/// ...) whose size is that of its first generic argument.
fn scalar_size(ty: &str) -> Result<u64, String> {
    let ty = ty.trim();
    match ty {
        "u8" => return Ok(1),
        "u16" => return Ok(2),
        "u32" => return Ok(4),
        "u64" => return Ok(8),
        _ => {}
    }

    let lt = ty
        .find('<')
        .ok_or_else(|| format!("unrecognized register type `{}`", ty))?;
    if !ty.ends_with('>') {
        return Err(format!("unrecognized register type `{}`", ty));
    }
    let inner = &ty[lt + 1..ty.len() - 1];
    let first_arg_end = find_top_level_char(inner, ',').unwrap_or(inner.len());
    let first_arg = inner[..first_arg_end].trim();
    scalar_size(first_arg)
}

/// A tiny recursive-descent evaluator for integer expressions made of
/// literals (decimal or `0x` hex), `+`, `-`, `*`, `/`, and parens. Does not
/// resolve named constants.
fn eval_int_expr(s: &str) -> Result<u64, String> {
    let tokens = tokenize_expr(s)?;
    let mut pos = 0;
    let value = parse_expr(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(format!("unexpected trailing input in `{}`", s));
    }
    Ok(value)
}

#[derive(Debug, Clone)]
enum Tok {
    Num(u64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize_expr(s: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            c if c.is_ascii_digit() => {
                let start = i;
                if c == '0' && chars.get(i + 1).map(|c| *c == 'x' || *c == 'X').unwrap_or(false) {
                    i += 2;
                    let hex_start = i;
                    while i < chars.len() && chars[i].is_ascii_hexdigit() {
                        i += 1;
                    }
                    let digits: String = chars[hex_start..i].iter().collect();
                    let n = u64::from_str_radix(&digits, 16)
                        .map_err(|_| format!("bad hex literal in `{}`", s))?;
                    toks.push(Tok::Num(n));
                } else {
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                        i += 1;
                    }
                    let digits: String = chars[start..i].iter().filter(|c| **c != '_').collect();
                    let n: u64 = digits
                        .parse()
                        .map_err(|_| format!("bad integer literal in `{}`", s))?;
                    toks.push(Tok::Num(n));
                }
            }
            other => {
                return Err(format!(
                    "unexpected character `{}` (named constants aren't supported)",
                    other
                ));
            }
        }
    }
    Ok(toks)
}

fn parse_expr(toks: &[Tok], pos: &mut usize) -> Result<u64, String> {
    let mut value = parse_term(toks, pos)?;
    loop {
        match toks.get(*pos) {
            Some(Tok::Plus) => {
                *pos += 1;
                value += parse_term(toks, pos)?;
            }
            Some(Tok::Minus) => {
                *pos += 1;
                value -= parse_term(toks, pos)?;
            }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_term(toks: &[Tok], pos: &mut usize) -> Result<u64, String> {
    let mut value = parse_factor(toks, pos)?;
    loop {
        match toks.get(*pos) {
            Some(Tok::Star) => {
                *pos += 1;
                value *= parse_factor(toks, pos)?;
            }
            Some(Tok::Slash) => {
                *pos += 1;
                let rhs = parse_factor(toks, pos)?;
                value /= rhs;
            }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_factor(toks: &[Tok], pos: &mut usize) -> Result<u64, String> {
    match toks.get(*pos) {
        Some(Tok::Num(n)) => {
            *pos += 1;
            Ok(*n)
        }
        Some(Tok::LParen) => {
            *pos += 1;
            let v = parse_expr(toks, pos)?;
            match toks.get(*pos) {
                Some(Tok::RParen) => {
                    *pos += 1;
                    Ok(v)
                }
                _ => Err("missing closing paren".to_string()),
            }
        }
        other => Err(format!("unexpected token {:?}", other)),
    }
}

/// Ensures `register_structs` is imported from the same path the file
/// already imports its register wrapper types from (defaulting to
/// `kernel::utilities::registers`), so the macro invocation compiles.
fn add_register_structs_import(text: &str) -> String {
    if let Some((_line_start, brace_pos, close_pos)) = find_registers_use_brace(text) {
        let items_text = &text[brace_pos + 1..close_pos];
        if items_text.split(',').any(|i| i.trim() == "register_structs") {
            return text.to_string();
        }
        let mut items: Vec<String> = items_text
            .split(',')
            .map(|i| i.trim().to_string())
            .filter(|i| !i.is_empty())
            .collect();
        items.push("register_structs".to_string());
        items.sort();
        let new_items = items.join(", ");
        return format!(
            "{}{{{}}}{}",
            &text[..brace_pos],
            new_items,
            &text[close_pos + 1..]
        );
    }

    // No existing `::registers::{...}` use block; try a single-item use of
    // one of the register wrapper types and widen it to a brace list.
    for ty in ["ReadWrite", "ReadOnly", "WriteOnly"] {
        let pat = format!("::registers::{};", ty);
        if let Some(p) = text.find(&pat) {
            let before = &text[..p];
            let after = &text[p + pat.len()..];
            return format!("{}::registers::{{{}, register_structs}};{}", before, ty, after);
        }
    }

    // Fall back: insert a new `use` line after the last existing leading
    // `use` statement.
    if let Some(last_use_end) = last_leading_use_end(text) {
        format!(
            "{}use kernel::utilities::registers::register_structs;\n{}",
            &text[..last_use_end],
            &text[last_use_end..]
        )
    } else {
        text.to_string()
    }
}

/// Locates a `use ...::registers::{ ... };` statement, returning
/// `(line_start, brace_open_pos, brace_close_pos)`.
fn find_registers_use_brace(text: &str) -> Option<(usize, usize, usize)> {
    let needle = "::registers::{";
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(needle) {
        let pos = search_from + rel;
        let line_start = line_start_at(text, pos);
        let line_end = text[pos..].find('\n').map(|i| pos + i).unwrap_or(text.len());
        let full_line = &text[line_start..line_end];
        if full_line.trim_start().starts_with("use ") {
            let brace_pos = pos + needle.len() - 1;
            if let Some(close) = find_matching_brace(text, brace_pos) {
                return Some((line_start, brace_pos, close));
            }
        }
        search_from = pos + needle.len();
    }
    None
}

/// Finds the end of the last `use ...;` statement among the leading import
/// block of the file (before any non-use, non-comment, non-attribute,
/// non-blank line).
fn last_leading_use_end(text: &str) -> Option<usize> {
    let mut last_end = None;
    let mut pos = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        let line_end = (pos + line.len() + 1).min(text.len());
        if trimmed.starts_with("use ") {
            last_end = Some(line_end);
        } else if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("#!")
            || trimmed.starts_with('#')
        {
            // Allowed before/between use statements; keep scanning.
        } else if last_end.is_some() {
            break;
        }
        pos = line_end;
    }
    last_end
}

#[cfg(test)]
mod tests {
    use super::*;

    const GPIO_SRC: &str = r#"use kernel::utilities::StaticRef;
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly};

/// General-purpose I/Os
#[repr(C)]
struct GpioRegisters {
    // GPIO data register
    dr: ReadWrite<u32>,
    // GPIO direction register
    gdir: ReadWrite<u32>,
    // GPIO pad status register
    psr: ReadOnly<u32>,
    // GPIO Interrupt configuration register 1
    icr1: ReadWrite<u32>,
    // GPIO Interrupt configuration register 2
    icr2: ReadWrite<u32>,
    // GPIO interrupt mask register
    imr: ReadWrite<u32>,
    // GPIO interrupt status register -- W1C - Write 1 to clear
    isr: ReadWrite<u32>,
    // GPIO edge select register
    edge_sel: ReadWrite<u32>,
    _reserved1: [u8; 100],
    // GPIO data register set
    dr_set: WriteOnly<u32>,
    // GPIO data register clear
    dr_clear: WriteOnly<u32>,
    // GPIO data register toggle
    dr_toggle: WriteOnly<u32>,
}

const GPIO1_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401B8000 as *const GpioRegisters) };
"#;

    #[test]
    fn converts_imxrt_gpio_struct() {
        let Conversion {
            output,
            structs_converted,
        } = convert_file(GPIO_SRC).unwrap();
        assert_eq!(structs_converted, 1);
        println!("{output}");

        assert!(output.contains("register_structs! {"));
        assert!(output.contains("/// General-purpose I/Os"));
        assert!(output.contains("GpioRegisters {"));
        assert!(output.contains("/// GPIO data register"));
        assert!(output.contains("(0x000 => dr: ReadWrite<u32>),"));
        assert!(output.contains("(0x004 => gdir: ReadWrite<u32>),"));
        assert!(output.contains("(0x008 => psr: ReadOnly<u32>),"));
        assert!(output.contains("(0x01c => edge_sel: ReadWrite<u32>),"));
        assert!(output.contains("(0x020 => _reserved1),"));
        // 0x020 + 100 (0x64) = 0x084
        assert!(output.contains("(0x084 => dr_set: WriteOnly<u32>),"));
        assert!(output.contains("(0x088 => dr_clear: WriteOnly<u32>),"));
        assert!(output.contains("(0x08c => dr_toggle: WriteOnly<u32>),"));
        assert!(output.contains("(0x090 => @END),"));
        assert!(output.contains(
            "use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_structs};"
        ));
    }

    #[test]
    fn handles_bitfield_generics_and_consts() {
        let src = r#"use kernel::utilities::registers::ReadWrite;

#[repr(C)]
struct FooRegisters {
    ctrl: ReadWrite<u32, CTRL::Register>,
    _reserved: [u8; 0x10 - 0x4],
    data: [ReadWrite<u16>; 4],
}
"#;
        let Conversion { output, .. } = convert_file(src).unwrap();
        println!("{output}");
        assert!(output.contains("(0x000 => ctrl: ReadWrite<u32, CTRL::Register>),"));
        assert!(output.contains("(0x004 => _reserved),"));
        assert!(output.contains("(0x010 => data: [ReadWrite<u16>; 4]),"));
        assert!(output.contains("(0x018 => @END),"));
    }

    #[test]
    fn rejects_unknown_type() {
        let src = r#"
#[repr(C)]
struct FooRegisters {
    thing: SomeWeirdType,
}
"#;
        assert!(convert_file(src).is_err());
    }
}
