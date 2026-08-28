//! Walks a Rust crate and extracts every code element (function, struct,
//! enum, module, ...) that carries a `# Code Tier` doc comment, writing the
//! result as a YAML list to a file. This is all this tool does -- it does
//! not look at how code calls other code; see `tier-checker` for that.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::Serialize;

/// The named assurance levels, in order from strongest to weakest guarantee.
const ASSURANCE_LEVELS: &[&str] = &[
    "Formally Verified",
    "Extensively Tested",
    "Functionally Tested",
    "Normal",
];

/// The named importance levels, in order from most to least critical.
const IMPORTANCE_LEVELS: &[&str] = &["Critical", "Widely Used", "Normal", "Experimental"];

/// A named level (e.g. "Extensively Tested") and its 1-based rank within
/// its category (1 is strongest/most critical).
#[derive(Serialize)]
struct Level {
    tier: String,
    rank: u32,
}

/// The assurance and importance levels extracted from a `# Code Tier`
/// doc comment.
#[derive(Serialize)]
struct CodeLevel {
    assurance: Level,
    importance: Level,
}

/// The rank (1-based position) of `value` within `levels`, if it names one
/// of the known levels.
fn level_rank(levels: &[&str], value: &str) -> Option<u32> {
    levels
        .iter()
        .position(|level| *level == value)
        .map(|i| i as u32 + 1)
}

/// One item in the crate (a file, module, function, struct, ...) that has
/// a `# Code Tier` doc comment.
#[derive(Serialize)]
struct Entry {
    file: String,
    /// The formal Rust path to the item, e.g. `kernel::debug::PanicResources`.
    path: String,
    kind: String,
    #[serde(flatten)]
    code_level: CodeLevel,
}

/// Concatenate all `#[doc = "..."]` attributes (i.e. `///` and `//!`
/// comments) attached to an item into a single string, one line per
/// attribute.
fn get_doc_string(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if let syn::Meta::NameValue(syn::MetaNameValue { path, value, .. }) = &attr.meta
            && path.is_ident("doc")
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = value
        {
            lines.push(s.value());
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Look for a `# Code Tier` markdown header in an item's doc comment and,
/// if found, parse the `Assurance` and `Importance` list items beneath it.
///
/// ```text
/// /// # Code Tier
/// ///
/// /// - Assurance: Normal
/// /// - Importance: Widely Used
/// ```
fn get_code_level(attrs: &[syn::Attribute]) -> Option<CodeLevel> {
    let doc = get_doc_string(attrs)?;
    let lines: Vec<&str> = doc.lines().map(|l| l.trim()).collect();
    let header = lines.iter().position(|l| *l == "# Code Tier")?;

    let mut assurance = None;
    let mut importance = None;
    for line in &lines[header + 1..] {
        if line.is_empty() {
            continue;
        }
        // Stop at the next markdown header.
        if line.starts_with('#') {
            break;
        }
        if let Some(rest) = line.strip_prefix("- Assurance:") {
            let value = rest.trim();
            match level_rank(ASSURANCE_LEVELS, value) {
                Some(rank) => {
                    assurance = Some(Level {
                        tier: value.to_string(),
                        rank,
                    })
                }
                None => eprintln!("warning: unrecognized assurance level {value:?}"),
            }
        } else if let Some(rest) = line.strip_prefix("- Importance:") {
            let value = rest.trim();
            match level_rank(IMPORTANCE_LEVELS, value) {
                Some(rank) => {
                    importance = Some(Level {
                        tier: value.to_string(),
                        rank,
                    })
                }
                None => eprintln!("warning: unrecognized importance level {value:?}"),
            }
        }
    }

    match (assurance, importance) {
        (Some(assurance), Some(importance)) => Some(CodeLevel {
            assurance,
            importance,
        }),
        _ => None,
    }
}

/// Join a module path and an item name into a single `::`-separated path.
fn join_path(module_path: &str, name: &str) -> String {
    if module_path.is_empty() {
        name.to_string()
    } else {
        format!("{module_path}::{name}")
    }
}

/// Render a `syn::Type` (e.g. the self type of an `impl` block) back to
/// source text.
fn type_name(ty: &syn::Type) -> String {
    quote::quote!(#ty).to_string()
}

/// The value of a `#[path = "..."]` attribute on a `mod` item, if present.
fn mod_path_attr(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let syn::Meta::NameValue(syn::MetaNameValue { path, value, .. }) = &attr.meta
            && path.is_ident("path")
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = value
        {
            return Some(s.value());
        }
    }
    None
}

/// Whether an item is gated behind `#[cfg(test)]` and should be excluded
/// from the crate walk entirely -- it's test-only code, not part of the
/// crate's real element set.
fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_args::<syn::Path>()
                .is_ok_and(|p| p.is_ident("test"))
    })
}

/// The directory that child modules of `file` live in, following normal
/// Rust module-file conventions (`foo.rs` -> `foo/`, `mod.rs`/`lib.rs`/
/// `main.rs` -> their own directory).
fn submodule_dir(file: &Path) -> PathBuf {
    let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let dir = file.parent().unwrap_or_else(|| Path::new(""));
    if matches!(file_name, "mod.rs" | "lib.rs" | "main.rs") {
        dir.to_path_buf()
    } else {
        dir.join(file.file_stem().unwrap_or_default())
    }
}

/// Find the on-disk file for a `mod name;` declaration in `file`.
fn resolve_mod_file(file: &Path, name: &str, attrs: &[syn::Attribute]) -> Option<PathBuf> {
    if let Some(explicit) = mod_path_attr(attrs) {
        let dir = file.parent().unwrap_or_else(|| Path::new(""));
        return dir.join(explicit).canonicalize().ok();
    }
    let dir = submodule_dir(file);
    dir.join(format!("{name}.rs"))
        .canonicalize()
        .ok()
        .or_else(|| dir.join(name).join("mod.rs").canonicalize().ok())
}

/// `path` relative to `base` (e.g. the crate's parent directory), so a
/// file reads like `kernel/src/grant.rs` instead of an absolute path.
/// Falls back to the absolute path if `path` isn't under `base`.
fn relative_file(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

/// Best-effort extraction of the `name` field from a `[package]` table in
/// a `Cargo.toml` file, without pulling in a full TOML parser.
fn crate_name_from_cargo_toml(cargo_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    let mut in_package = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = line.strip_prefix("name")
            && let Some(rest) = rest.trim_start().strip_prefix('=')
        {
            let value = rest.trim().trim_matches('"');
            return Some(value.to_string());
        }
    }
    None
}

fn record(
    results: &mut Vec<Entry>,
    file: &str,
    module_path: &str,
    name: &str,
    kind: &str,
    attrs: &[syn::Attribute],
) {
    if let Some(code_level) = get_code_level(attrs) {
        results.push(Entry {
            file: file.to_string(),
            path: join_path(module_path, name),
            kind: kind.to_string(),
            code_level,
        });
    }
}

/// Mutable state threaded through the crate walk: the parsed source files
/// (immutable, borrowed for `'a`), and the accumulated annotated elements.
struct WalkState<'a> {
    files: &'a HashMap<PathBuf, syn::File>,
    visited: HashSet<PathBuf>,
    results: Vec<Entry>,
}

/// Recursively walk all items in a file (or module), recording any item
/// whose doc comment has a `# Code Tier` annotation. External module
/// declarations (`mod foo;`) are followed into their own file using
/// `state.files`, so that `module_path` always reflects the formal Rust
/// path to each item.
fn walk_items<'a>(
    items: &'a [syn::Item],
    file: &Path,
    module_path: &str,
    state: &mut WalkState<'a>,
) {
    let file_str = file.to_string_lossy().to_string();
    for item in items {
        match item {
            syn::Item::Fn(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.sig.ident.to_string(),
                    "fn",
                    &i.attrs,
                );
            }
            syn::Item::Struct(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "struct",
                    &i.attrs,
                );
            }
            syn::Item::Enum(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "enum",
                    &i.attrs,
                );
            }
            syn::Item::Union(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "union",
                    &i.attrs,
                );
            }
            syn::Item::Const(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "const",
                    &i.attrs,
                );
            }
            syn::Item::Static(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "static",
                    &i.attrs,
                );
            }
            syn::Item::Type(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "type",
                    &i.attrs,
                );
            }
            syn::Item::Trait(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                let name = i.ident.to_string();
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &name,
                    "trait",
                    &i.attrs,
                );
                let trait_path = join_path(module_path, &name);
                for trait_item in &i.items {
                    if let syn::TraitItem::Fn(f) = trait_item
                        && !is_cfg_test(&f.attrs)
                    {
                        record(
                            &mut state.results,
                            &file_str,
                            &trait_path,
                            &f.sig.ident.to_string(),
                            "trait_fn",
                            &f.attrs,
                        );
                    }
                }
            }
            syn::Item::Impl(i) => {
                if is_cfg_test(&i.attrs) {
                    continue;
                }
                let self_ty = type_name(&i.self_ty);
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &self_ty,
                    "impl",
                    &i.attrs,
                );
                let self_context = match &i.trait_ {
                    Some((_, trait_path, _)) => {
                        format!("<{self_ty} as {}>", quote::quote!(#trait_path))
                    }
                    None => self_ty.clone(),
                };
                let impl_path = join_path(module_path, &self_context);
                for impl_item in &i.items {
                    if let syn::ImplItem::Fn(f) = impl_item
                        && !is_cfg_test(&f.attrs)
                    {
                        record(
                            &mut state.results,
                            &file_str,
                            &impl_path,
                            &f.sig.ident.to_string(),
                            "impl_fn",
                            &f.attrs,
                        );
                    }
                }
            }
            syn::Item::Mod(i) => {
                let name = i.ident.to_string();
                if is_cfg_test(&i.attrs) {
                    // Don't record or recurse into a test-only module, but
                    // if it points at its own file, still mark that file
                    // visited -- otherwise the "unreachable file" fallback
                    // later would pick it up and include it anyway.
                    if i.content.is_none()
                        && let Some(sub_file) = resolve_mod_file(file, &name, &i.attrs)
                    {
                        state.visited.insert(sub_file);
                    }
                    continue;
                }
                record(
                    &mut state.results,
                    &file_str,
                    module_path,
                    &name,
                    "mod",
                    &i.attrs,
                );
                let mod_path = join_path(module_path, &name);
                if let Some((_, sub_items)) = &i.content {
                    // Inline module: `mod foo { ... }`.
                    walk_items(sub_items, file, &mod_path, state);
                } else if let Some(sub_file) = resolve_mod_file(file, &name, &i.attrs) {
                    // External module: `mod foo;`, defined in its own file.
                    if state.visited.insert(sub_file.clone())
                        && let Some(sub_ast) = state.files.get(&sub_file)
                    {
                        record(
                            &mut state.results,
                            &sub_file.to_string_lossy(),
                            "",
                            &mod_path,
                            "file",
                            &sub_ast.attrs,
                        );
                        walk_items(&sub_ast.items, &sub_file, &mod_path, state);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Walk a crate root starting at `entry_file` (e.g. `src/lib.rs`), whose
/// formal module path is `module_name`, following `mod` declarations into
/// their files as they're encountered.
fn walk_crate_root<'a>(entry_file: &Path, module_name: &str, state: &mut WalkState<'a>) {
    let Some(entry_file) = entry_file.canonicalize().ok() else {
        return;
    };
    let Some(ast) = state.files.get(&entry_file) else {
        return;
    };
    if !state.visited.insert(entry_file.clone()) {
        return;
    }
    record(
        &mut state.results,
        &entry_file.to_string_lossy(),
        "",
        module_name,
        "file",
        &ast.attrs,
    );
    walk_items(&ast.items, &entry_file, module_name, state);
}

/// Extract the `members` list from a `[workspace]` table in a `Cargo.toml`
/// file, without pulling in a full TOML parser. Returns `None` if the file
/// has no `[workspace]` table with a `members` array (i.e. it's a plain
/// crate manifest, not a workspace root). Handles both
/// `members = ["a", "b"]` on one line and the more common style of one
/// quoted path per line.
fn workspace_members(cargo_toml: &Path) -> Option<Vec<String>> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    let mut in_workspace = false;
    let mut in_members = false;
    let mut members = Vec::new();

    fn collect_quoted(text: &str, out: &mut Vec<String>) {
        let mut rest = text;
        while let Some(start) = rest.find('"') {
            let after = &rest[start + 1..];
            let Some(end) = after.find('"') else { break };
            out.push(after[..end].to_string());
            rest = &after[end + 1..];
        }
    }

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_workspace = line == "[workspace]";
            in_members = false;
            continue;
        }
        if !in_workspace {
            continue;
        }
        if !in_members {
            let Some(rest) = line.strip_prefix("members") else {
                continue;
            };
            let Some(rest) = rest.trim_start().strip_prefix('=') else {
                continue;
            };
            let Some(rest) = rest.trim_start().strip_prefix('[') else {
                continue;
            };
            in_members = true;
            collect_quoted(rest, &mut members);
            if rest.contains(']') {
                in_members = false;
            }
        } else {
            collect_quoted(line, &mut members);
            if line.contains(']') {
                in_members = false;
            }
        }
    }

    if members.is_empty() {
        None
    } else {
        Some(members)
    }
}

/// Extract every `# Code Tier`-annotated element from a single crate
/// rooted at `crate_root` (a directory containing a `Cargo.toml` and a
/// `src/` directory). File paths in the returned entries are absolute;
/// the caller rewrites them relative to whatever base makes sense.
fn extract_crate(crate_root: &Path) -> Vec<Entry> {
    let crate_name =
        crate_name_from_cargo_toml(&crate_root.join("Cargo.toml")).unwrap_or_else(|| {
            crate_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    // Parse every `.rs` file in the crate up front so that `mod foo;`
    // declarations can be resolved to their file's contents.
    let mut files: HashMap<PathBuf, syn::File> = HashMap::new();
    for entry in WalkBuilder::new(crate_root).build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("warning: {err}");
                continue;
            }
        };

        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let Ok(canonical) = entry.path().canonicalize() else {
            continue;
        };
        let content = fs::read_to_string(entry.path()).unwrap();
        match syn::parse_file(&content) {
            Ok(ast) => {
                files.insert(canonical, ast);
            }
            Err(err) => {
                eprintln!("warning: failed to parse {}: {err}", entry.path().display());
            }
        }
    }

    let mut state = WalkState {
        files: &files,
        visited: HashSet::new(),
        results: Vec::new(),
    };

    // Formal crate roots: the library and/or binary entry points.
    let mut roots: Vec<(PathBuf, String)> = vec![
        (crate_root.join("src/lib.rs"), crate_name.clone()),
        (crate_root.join("src/main.rs"), crate_name.clone()),
    ];
    for pattern_dir in ["src/bin", "examples", "tests", "benches"] {
        let Ok(read_dir) = fs::read_dir(crate_root.join(pattern_dir)) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                roots.push((path, name));
            }
        }
    }

    for (entry_file, module_name) in &roots {
        walk_crate_root(entry_file, module_name, &mut state);
    }

    // Any parsed file that wasn't reached through module resolution (e.g.
    // it isn't actually wired into the crate via `mod`) is still scanned,
    // using a best-effort path derived from its location on disk.
    let mut unresolved: Vec<PathBuf> = files
        .keys()
        .filter(|f| !state.visited.contains(*f))
        .cloned()
        .collect();
    unresolved.sort();
    for path in unresolved {
        // May have been pulled in via `mod` from another unresolved file
        // processed earlier in this loop.
        if !state.visited.insert(path.clone()) {
            continue;
        }
        eprintln!(
            "warning: {} is not reachable from a crate root via `mod`; using a best-effort path",
            path.display()
        );
        let relative = path.strip_prefix(crate_root).unwrap_or(&path);
        let guessed_path = relative
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("::");
        let ast = &files[&path];
        record(
            &mut state.results,
            &path.to_string_lossy(),
            "",
            &guessed_path,
            "file",
            &ast.attrs,
        );
        walk_items(&ast.items, &path, &guessed_path, &mut state);
    }

    state.results
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("usage: tier-extractor <path to crate or workspace> <output.yaml>");
    }

    let root = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let root_canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    // Files are recorded with an absolute path during each crate's walk
    // (so `mod` resolution and cfg(test) file-tracking can canonicalize
    // freely); at the end they're rewritten relative to `file_base`, so
    // e.g. `/Users/.../tock/kernel/src/grant.rs` becomes
    // `kernel/src/grant.rs`.
    let (crate_roots, file_base): (Vec<PathBuf>, PathBuf) =
        match workspace_members(&root.join("Cargo.toml")) {
            Some(members) => {
                eprintln!(
                    "processing workspace with {} member crate(s)",
                    members.len()
                );
                let crate_roots = members.iter().map(|m| root.join(m)).collect();
                (crate_roots, root_canonical.clone())
            }
            None => {
                let file_base = root_canonical
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| root_canonical.clone());
                (vec![root.to_path_buf()], file_base)
            }
        };

    let mut results: Vec<Entry> = Vec::new();
    for crate_root in &crate_roots {
        if !crate_root.join("Cargo.toml").is_file() {
            eprintln!(
                "warning: {} has no Cargo.toml; skipping",
                crate_root.display()
            );
            continue;
        }
        let crate_results = extract_crate(crate_root);
        eprintln!(
            "  {}: {} annotated element(s)",
            crate_root.display(),
            crate_results.len()
        );
        results.extend(crate_results);
    }

    for entry in &mut results {
        entry.file = relative_file(Path::new(&entry.file), &file_base);
    }

    eprintln!(
        "extracted {} annotated element(s) from {} crate(s)",
        results.len(),
        crate_roots.len()
    );

    let yaml = serde_yaml::to_string(&results).unwrap();
    if let Err(err) = fs::write(output_path, yaml) {
        panic!("failed to write {}: {err}", output_path.display());
    }
}
