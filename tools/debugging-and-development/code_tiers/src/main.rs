use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::Serialize;

/// The assurance and criticality numbers extracted from a `# Code Level`
/// doc comment.
#[derive(Serialize)]
struct CodeLevel {
    assurance: u32,
    criticality: u32,
}

/// One item in the crate (a file, module, function, struct, ...) that has
/// a `# Code Level` doc comment.
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

/// Look for a `# Code Level` markdown header in an item's doc comment and,
/// if found, parse the `Assurance` and `Criticality` list items beneath it.
///
/// ```text
/// /// # Code Level
/// ///
/// /// - Assurance: 3
/// /// - Criticality: 1
/// ```
fn get_code_level(attrs: &[syn::Attribute]) -> Option<CodeLevel> {
    let doc = get_doc_string(attrs)?;
    let lines: Vec<&str> = doc.lines().map(|l| l.trim()).collect();
    let header = lines.iter().position(|l| *l == "# Code Level")?;

    let mut assurance = None;
    let mut criticality = None;
    for line in &lines[header + 1..] {
        if line.is_empty() {
            continue;
        }
        // Stop at the next markdown header.
        if line.starts_with('#') {
            break;
        }
        if let Some(rest) = line.strip_prefix("- Assurance:") {
            assurance = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("- Criticality:") {
            criticality = rest.trim().parse().ok();
        }
    }

    match (assurance, criticality) {
        (Some(assurance), Some(criticality)) => Some(CodeLevel {
            assurance,
            criticality,
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

/// Recursively walk all items in a file (or module), recording any item
/// whose doc comment has a `# Code Level` annotation. External module
/// declarations (`mod foo;`) are followed into their own file using
/// `files` (the set of all parsed `.rs` files in the crate), so that
/// `module_path` always reflects the formal Rust path to each item.
#[allow(clippy::too_many_arguments)]
fn walk_items(
    items: &[syn::Item],
    file: &Path,
    module_path: &str,
    files: &HashMap<PathBuf, syn::File>,
    visited: &mut HashSet<PathBuf>,
    results: &mut Vec<Entry>,
) {
    let file_str = file.to_string_lossy().to_string();
    for item in items {
        match item {
            syn::Item::Fn(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.sig.ident.to_string(),
                    "fn",
                    &i.attrs,
                );
            }
            syn::Item::Struct(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "struct",
                    &i.attrs,
                );
            }
            syn::Item::Enum(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "enum",
                    &i.attrs,
                );
            }
            syn::Item::Union(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "union",
                    &i.attrs,
                );
            }
            syn::Item::Const(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "const",
                    &i.attrs,
                );
            }
            syn::Item::Static(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "static",
                    &i.attrs,
                );
            }
            syn::Item::Type(i) => {
                record(
                    results,
                    &file_str,
                    module_path,
                    &i.ident.to_string(),
                    "type",
                    &i.attrs,
                );
            }
            syn::Item::Trait(i) => {
                let name = i.ident.to_string();
                record(results, &file_str, module_path, &name, "trait", &i.attrs);
                let trait_path = join_path(module_path, &name);
                for trait_item in &i.items {
                    if let syn::TraitItem::Fn(f) = trait_item {
                        record(
                            results,
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
                let name = type_name(&i.self_ty);
                record(results, &file_str, module_path, &name, "impl", &i.attrs);
                let impl_path = join_path(module_path, &name);
                for impl_item in &i.items {
                    if let syn::ImplItem::Fn(f) = impl_item {
                        record(
                            results,
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
                record(results, &file_str, module_path, &name, "mod", &i.attrs);
                let mod_path = join_path(module_path, &name);
                if let Some((_, sub_items)) = &i.content {
                    // Inline module: `mod foo { ... }`.
                    walk_items(sub_items, file, &mod_path, files, visited, results);
                } else if let Some(sub_file) = resolve_mod_file(file, &name, &i.attrs) {
                    // External module: `mod foo;`, defined in its own file.
                    if visited.insert(sub_file.clone())
                        && let Some(sub_ast) = files.get(&sub_file)
                    {
                        record(
                            results,
                            &sub_file.to_string_lossy(),
                            "",
                            &mod_path,
                            "file",
                            &sub_ast.attrs,
                        );
                        walk_items(
                            &sub_ast.items,
                            &sub_file,
                            &mod_path,
                            files,
                            visited,
                            results,
                        );
                    }
                }
            }
            _ => {}
        }
    }
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

/// Walk a crate root starting at `entry_file` (e.g. `src/lib.rs`), whose
/// formal module path is `crate_name`, following `mod` declarations into
/// their files as they're encountered.
fn walk_crate_root(
    entry_file: &Path,
    crate_name: &str,
    files: &HashMap<PathBuf, syn::File>,
    visited: &mut HashSet<PathBuf>,
    results: &mut Vec<Entry>,
) {
    let Some(entry_file) = entry_file.canonicalize().ok() else {
        return;
    };
    let Some(ast) = files.get(&entry_file) else {
        return;
    };
    if !visited.insert(entry_file.clone()) {
        return;
    }
    record(
        results,
        &entry_file.to_string_lossy(),
        "",
        crate_name,
        "file",
        &ast.attrs,
    );
    walk_items(&ast.items, &entry_file, crate_name, files, visited, results);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("must specify path to the crate to check");
    }

    let root = Path::new(&args[1]);
    let crate_name = crate_name_from_cargo_toml(&root.join("Cargo.toml")).unwrap_or_else(|| {
        root.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    // Parse every `.rs` file in the crate up front so that `mod foo;`
    // declarations can be resolved to their file's contents.
    let mut files: HashMap<PathBuf, syn::File> = HashMap::new();
    for entry in WalkBuilder::new(root).build() {
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

    let mut results = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();

    // Formal crate roots: the library and/or binary entry points.
    let mut roots: Vec<(PathBuf, String)> = vec![
        (root.join("src/lib.rs"), crate_name.clone()),
        (root.join("src/main.rs"), crate_name.clone()),
    ];
    for pattern_dir in ["src/bin", "examples", "tests", "benches"] {
        let Ok(read_dir) = fs::read_dir(root.join(pattern_dir)) else {
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
        walk_crate_root(entry_file, module_name, &files, &mut visited, &mut results);
    }

    // Any parsed file that wasn't reached through module resolution (e.g.
    // it isn't actually wired into the crate via `mod`) is still scanned,
    // using a best-effort path derived from its location on disk.
    let mut unresolved: Vec<PathBuf> = files
        .keys()
        .filter(|f| !visited.contains(*f))
        .cloned()
        .collect();
    unresolved.sort();
    for path in unresolved {
        // May have been pulled in via `mod` from another unresolved file
        // processed earlier in this loop.
        if !visited.insert(path.clone()) {
            continue;
        }
        eprintln!(
            "warning: {} is not reachable from a crate root via `mod`; using a best-effort path",
            path.display()
        );
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let guessed_path = relative
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("::");
        let ast = &files[&path];
        record(
            &mut results,
            &path.to_string_lossy(),
            "",
            &guessed_path,
            "file",
            &ast.attrs,
        );
        walk_items(
            &ast.items,
            &path,
            &guessed_path,
            &files,
            &mut visited,
            &mut results,
        );
    }

    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
