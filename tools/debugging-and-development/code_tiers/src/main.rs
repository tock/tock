use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::Serialize;
use syn::visit::Visit;

/// The named assurance levels, in order from strongest to weakest guarantee.
const ASSURANCE_LEVELS: &[&str] = &[
    "Formally Verified",
    "Extensively Tested",
    "Functionally Tested",
    "Normal",
];

/// The named importance levels, in order from most to least critical.
const IMPORTANCE_LEVELS: &[&str] = &["Critical", "Widely Used", "Normal", "Experimental"];

/// The assurance and importance levels extracted from a `# Code Tier`
/// doc comment, along with each level's rank (1 is strongest/most
/// critical) within its category.
#[derive(Serialize, Clone)]
struct CodeLevel {
    assurance: String,
    assurance_rank: u32,
    importance: String,
    importance_rank: u32,
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

/// A function-like item (`fn`, an `impl` method, or a `trait` method) found
/// anywhere in the crate, whether or not it carries a `# Code Tier`
/// annotation. These are the nodes of the call graph.
struct FunctionInfo<'a> {
    /// Formal Rust path, e.g. `kernel::debug::PanicResources::fmt`.
    path: String,
    kind: &'static str,
    file: String,
    /// The module the function is declared in (not including its `impl`/
    /// `trait` container, if any) -- this is the scope bare and `self::`/
    /// `super::` calls in its body are resolved against.
    module_path: String,
    /// For `impl` methods, the bare `Self` type name, used to resolve
    /// `Self::other_method()` calls.
    self_type_bare: Option<String>,
    body: Option<&'a syn::Block>,
    code_level: Option<CodeLevel>,
}

/// A call graph node, as emitted in the JSON output.
#[derive(Serialize)]
struct Node {
    path: String,
    kind: String,
    file: String,
    assurance: Option<String>,
    assurance_rank: Option<u32>,
    importance: Option<String>,
    importance_rank: Option<u32>,
}

/// A call graph edge: `caller` calls `raw` (the callee as written in
/// source), which was resolved to `callee` if unambiguous, or to one of
/// several `candidates` if ambiguous (best-effort: this tool does not do
/// full type inference, so method calls in particular are matched by name
/// only).
#[derive(Serialize)]
struct Edge {
    caller: String,
    raw: String,
    resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    callee: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    candidates: Vec<String>,
}

#[derive(Serialize)]
struct CallGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Serialize)]
struct Output {
    annotations: Vec<Entry>,
    call_graph: CallGraph,
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
                Some(rank) => assurance = Some((value.to_string(), rank)),
                None => eprintln!("warning: unrecognized assurance level {value:?}"),
            }
        } else if let Some(rest) = line.strip_prefix("- Importance:") {
            let value = rest.trim();
            match level_rank(IMPORTANCE_LEVELS, value) {
                Some(rank) => importance = Some((value.to_string(), rank)),
                None => eprintln!("warning: unrecognized importance level {value:?}"),
            }
        }
    }

    match (assurance, importance) {
        (Some((assurance, assurance_rank)), Some((importance, importance_rank))) => {
            Some(CodeLevel {
                assurance,
                assurance_rank,
                importance,
                importance_rank,
            })
        }
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

/// The last `::`-separated component of a formal path.
fn simple_name(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
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
/// crate's real call graph.
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

/// Record a function-like item (`fn`, `impl` method, or `trait` method) as
/// a call graph node, and -- if it carries a `# Code Tier` annotation --
/// also as an annotation entry, matching the behavior of `record` for
/// other item kinds.
#[allow(clippy::too_many_arguments)]
fn record_function<'a>(
    state: &mut WalkState<'a>,
    file_str: &str,
    container_path: &str,
    module_path: &str,
    name: &str,
    kind: &'static str,
    self_type_bare: Option<String>,
    attrs: &[syn::Attribute],
    body: Option<&'a syn::Block>,
) {
    let path = join_path(container_path, name);
    let code_level = get_code_level(attrs);
    if let Some(level) = code_level.clone() {
        state.results.push(Entry {
            file: file_str.to_string(),
            path: path.clone(),
            kind: kind.to_string(),
            code_level: level,
        });
    }
    state.functions.push(FunctionInfo {
        path,
        kind,
        file: file_str.to_string(),
        module_path: module_path.to_string(),
        self_type_bare,
        body,
        code_level,
    });
}

/// A `::`-joined suffix built from path segments, e.g. `["foo", "bar"]` ->
/// `"::foo::bar"`, or `""` for an empty slice.
fn suffix_of(segments: &[String]) -> String {
    if segments.is_empty() {
        String::new()
    } else {
        format!("::{}", segments.join("::"))
    }
}

/// Resolve a leading `crate`/`self`/`super` segment (as used in both `use`
/// declarations and call expressions) against `module_path`, producing an
/// absolute-ish formal path. Any other leading segment is left as-is
/// (either an external crate, or -- treated as a same-crate uniform path --
/// resolved relative to the crate root by the caller).
fn substitute_absolute(segments: &[String], module_path: &str, crate_name: &str) -> String {
    if segments.is_empty() {
        return String::new();
    }
    match segments[0].as_str() {
        "crate" => format!("{crate_name}{}", suffix_of(&segments[1..])),
        "self" => format!("{module_path}{}", suffix_of(&segments[1..])),
        "super" => {
            let mut mp: Vec<&str> = module_path.split("::").collect();
            let mut idx = 0;
            while segments.get(idx).map(|s| s == "super").unwrap_or(false) {
                mp.pop();
                idx += 1;
            }
            format!("{}{}", mp.join("::"), suffix_of(&segments[idx..]))
        }
        _ => segments.join("::"),
    }
}

/// Flatten a `use` tree into `out`, mapping each locally-bound name to its
/// (best-effort) formal path. Glob imports (`use foo::*;`) can't be
/// resolved this way and are skipped.
fn flatten_use_tree(
    prefix: &[String],
    tree: &syn::UseTree,
    module_path: &str,
    crate_name: &str,
    out: &mut HashMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(p) => {
            let mut prefix = prefix.to_vec();
            prefix.push(p.ident.to_string());
            flatten_use_tree(&prefix, &p.tree, module_path, crate_name, out);
        }
        syn::UseTree::Name(n) => {
            let ident = n.ident.to_string();
            if ident == "self" {
                if let Some(local) = prefix.last() {
                    out.insert(
                        local.clone(),
                        substitute_absolute(prefix, module_path, crate_name),
                    );
                }
            } else {
                let mut full = prefix.to_vec();
                full.push(ident.clone());
                out.insert(ident, substitute_absolute(&full, module_path, crate_name));
            }
        }
        syn::UseTree::Rename(r) => {
            let mut full = prefix.to_vec();
            full.push(r.ident.to_string());
            out.insert(
                r.rename.to_string(),
                substitute_absolute(&full, module_path, crate_name),
            );
        }
        syn::UseTree::Glob(_) => {}
        syn::UseTree::Group(g) => {
            for item in &g.items {
                flatten_use_tree(prefix, item, module_path, crate_name, out);
            }
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

/// Mutable state threaded through the crate walk: the parsed source files
/// (immutable, borrowed for `'a`), and the accumulators the walk fills in.
struct WalkState<'a> {
    files: &'a HashMap<PathBuf, syn::File>,
    crate_name: String,
    visited: HashSet<PathBuf>,
    results: Vec<Entry>,
    functions: Vec<FunctionInfo<'a>>,
    /// Per-file map of locally-bound `use` names to their formal path.
    use_imports: HashMap<String, HashMap<String, String>>,
}

/// Recursively walk all items in a file (or module), recording any item
/// whose doc comment has a `# Code Tier` annotation, and registering
/// every function-like item as a call graph node. External module
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
                record_function(
                    state,
                    &file_str,
                    module_path,
                    module_path,
                    &i.sig.ident.to_string(),
                    "fn",
                    None,
                    &i.attrs,
                    Some(&i.block),
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
                        record_function(
                            state,
                            &file_str,
                            &trait_path,
                            module_path,
                            &f.sig.ident.to_string(),
                            "trait_fn",
                            None,
                            &f.attrs,
                            f.default.as_ref(),
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
                        record_function(
                            state,
                            &file_str,
                            &impl_path,
                            module_path,
                            &f.sig.ident.to_string(),
                            "impl_fn",
                            Some(self_ty.clone()),
                            &f.attrs,
                            Some(&f.block),
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
            syn::Item::Use(u) => {
                let entry = state.use_imports.entry(file_str.clone()).or_default();
                flatten_use_tree(&[], &u.tree, module_path, &state.crate_name, entry);
            }
            _ => {}
        }
    }
}

/// Walk a crate root starting at `entry_file` (e.g. `src/lib.rs`), whose
/// formal module path is `crate_name`, following `mod` declarations into
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

/// Collects the raw (unresolved) call expressions found in a function
/// body: plain calls (`foo()`, `a::b::foo()`) by their path segments, and
/// method calls (`x.foo()`) by method name.
#[derive(Default)]
struct CallCollector {
    calls: Vec<RawCall>,
}

enum RawCall {
    Path(Vec<String>),
    Method(String),
}

impl<'ast> Visit<'ast> for CallCollector {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*node.func {
            let segments: Vec<String> = p
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if !segments.is_empty() {
                self.calls.push(RawCall::Path(segments));
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.calls.push(RawCall::Method(node.method.to_string()));
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Best-effort resolution of a plain call's path segments (already split
/// on `::`) to the call graph node(s) it could refer to. Returns the
/// indices into `functions` of every candidate match: empty if nothing
/// matched, one if unambiguous, more if ambiguous.
#[allow(clippy::too_many_arguments)]
fn resolve_call(
    raw_segments: &[String],
    module_path: &str,
    self_type_bare: Option<&str>,
    crate_name: &str,
    use_map: &HashMap<String, String>,
    by_full_path: &HashMap<String, usize>,
    by_name: &HashMap<String, Vec<usize>>,
) -> Vec<usize> {
    let first = raw_segments[0].as_str();
    let rest = &raw_segments[1..];

    let mut candidates: Vec<String> = Vec::new();
    match first {
        "crate" => candidates.push(format!("{crate_name}{}", suffix_of(rest))),
        "self" => candidates.push(format!("{module_path}{}", suffix_of(rest))),
        "super" => {
            let mut mp: Vec<&str> = module_path.split("::").collect();
            let mut idx = 0;
            while raw_segments.get(idx).map(|s| s == "super").unwrap_or(false) {
                mp.pop();
                idx += 1;
            }
            candidates.push(format!(
                "{}{}",
                mp.join("::"),
                suffix_of(&raw_segments[idx..])
            ));
        }
        "Self" => {
            if let Some(st) = self_type_bare {
                candidates.push(format!("{module_path}::{st}{}", suffix_of(rest)));
            }
        }
        _ => {
            if let Some(mapped) = use_map.get(first) {
                candidates.push(format!("{mapped}{}", suffix_of(rest)));
            }
            candidates.push(format!("{module_path}::{}", raw_segments.join("::")));
            candidates.push(format!("{crate_name}::{}", raw_segments.join("::")));
        }
    }

    for c in &candidates {
        if let Some(&idx) = by_full_path.get(c) {
            return vec![idx];
        }
    }

    // Suffix match: does any known formal path end with this call path?
    let suffix = format!("::{}", raw_segments.join("::"));
    let mut suffix_matches: Vec<usize> = by_full_path
        .iter()
        .filter(|(k, _)| k.ends_with(&suffix))
        .map(|(_, &v)| v)
        .collect();
    if !suffix_matches.is_empty() {
        suffix_matches.sort_unstable();
        suffix_matches.dedup();
        return suffix_matches;
    }

    // Last resort: match by simple name alone. Only do this for bare,
    // unqualified calls (`foo()`) -- for a qualified call (`Cell::new()`),
    // stripping the qualifier and matching any `new` in the crate produces
    // massive false-positive ambiguity, and if `Cell` really were a
    // same-crate type, the candidates built above already would have found
    // it.
    if raw_segments.len() == 1 {
        by_name.get(&raw_segments[0]).cloned().unwrap_or_default()
    } else {
        Vec::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn push_edge(
    edges: &mut Vec<Edge>,
    caller_path: &str,
    matches: Vec<usize>,
    functions: &[FunctionInfo],
    raw: String,
    resolved: &mut u32,
    ambiguous: &mut u32,
    unresolved: &mut u32,
) {
    match matches.len() {
        0 => *unresolved += 1,
        1 => {
            *resolved += 1;
            edges.push(Edge {
                caller: caller_path.to_string(),
                raw,
                resolution: "resolved".to_string(),
                callee: Some(functions[matches[0]].path.clone()),
                candidates: Vec::new(),
            });
        }
        _ => {
            *ambiguous += 1;
            edges.push(Edge {
                caller: caller_path.to_string(),
                raw,
                resolution: "ambiguous".to_string(),
                callee: None,
                candidates: matches.iter().map(|&i| functions[i].path.clone()).collect(),
            });
        }
    }
}

fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// A valid Graphviz subgraph name derived from a module path: alphanumerics
/// and underscores only.
fn dot_identifier(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Box fill color: green for a function that carries a `# Code Tier`
/// annotation, neutral gray otherwise.
fn node_fill_color(code_level: &Option<CodeLevel>) -> &'static str {
    match code_level {
        Some(_) => "#81c784",
        None => "#eeeeee",
    }
}

fn write_dot(nodes: &[FunctionInfo], edges: &[Edge], path: &Path) -> std::io::Result<()> {
    let mut out = String::from(
        "digraph calls {\n  rankdir=LR;\n  node [shape=box, style=filled, fontsize=10];\n\n",
    );

    // Only draw resolved edges (see below), so only keep nodes that are a
    // caller or callee of at least one -- an isolated function adds no
    // information to a call graph.
    let connected: HashSet<&str> = edges
        .iter()
        .filter_map(|e| {
            e.callee
                .as_deref()
                .map(|callee| [e.caller.as_str(), callee])
        })
        .flatten()
        .collect();

    let mut by_path: HashMap<&str, &FunctionInfo> = HashMap::new();
    for f in nodes {
        by_path.insert(f.path.as_str(), f);
    }

    // Group nodes by their enclosing module and draw each as a labeled
    // Graphviz cluster, so `dot`'s layout keeps related functions together
    // instead of scattering them across the whole graph.
    let mut by_module: BTreeMap<&str, Vec<&FunctionInfo>> = BTreeMap::new();
    for f in nodes.iter().filter(|f| connected.contains(f.path.as_str())) {
        by_module.entry(f.module_path.as_str()).or_default().push(f);
    }

    for (i, (module_path, funcs)) in by_module.iter().enumerate() {
        out.push_str(&format!(
            "  subgraph cluster_{i}_{} {{\n    label=\"{}\";\n    style=dashed;\n    fontsize=10;\n",
            dot_identifier(module_path),
            dot_escape(module_path)
        ));
        for f in funcs {
            let label = match &f.code_level {
                Some(cl) => format!("{}\\n{} / {}", f.path, cl.assurance, cl.importance),
                None => f.path.clone(),
            };
            out.push_str(&format!(
                "    \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n",
                dot_escape(&f.path),
                dot_escape(&label),
                node_fill_color(&f.code_level)
            ));
        }
        out.push_str("  }\n\n");
    }

    // Ambiguous calls (multiple same-named candidates, no type info to
    // pick between them) are omitted rather than drawn to every candidate,
    // which would make the graph mostly noise.
    for e in edges {
        let Some(callee) = &e.callee else { continue };

        // Flag a call into weaker assurance: a higher assurance_rank means
        // a *weaker* guarantee (1 = Formally Verified, 4 = Normal), so the
        // callee is a downgrade when its rank is numerically higher than
        // the caller's. Only meaningful when both ends are annotated.
        let weaker_assurance = by_path
            .get(e.caller.as_str())
            .zip(by_path.get(callee.as_str()))
            .and_then(|(caller, callee)| {
                Some((caller.code_level.as_ref()?, callee.code_level.as_ref()?))
            })
            .is_some_and(|(caller, callee)| callee.assurance_rank > caller.assurance_rank);

        let attrs = if weaker_assurance {
            " [color=\"#e53935\", penwidth=2]"
        } else {
            ""
        };
        out.push_str(&format!(
            "  \"{}\" -> \"{}\"{attrs};\n",
            dot_escape(&e.caller),
            dot_escape(callee)
        ));
    }

    out.push_str("}\n");
    fs::write(path, out)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("must specify path to the crate to check");
    }

    let root = Path::new(&args[1]);
    let mut dot_path: Option<&str> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--dot" && i + 1 < args.len() {
            dot_path = Some(&args[i + 1]);
            i += 2;
        } else {
            i += 1;
        }
    }

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

    let mut state = WalkState {
        files: &files,
        crate_name: crate_name.clone(),
        visited: HashSet::new(),
        results: Vec::new(),
        functions: Vec::new(),
        use_imports: HashMap::new(),
    };

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
        let relative = path.strip_prefix(root).unwrap_or(&path);
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

    let WalkState {
        results,
        functions,
        use_imports,
        ..
    } = state;

    // Build lookup indices over every function-like item in the crate.
    let mut by_full_path: HashMap<String, usize> = HashMap::new();
    let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, f) in functions.iter().enumerate() {
        by_full_path.insert(f.path.clone(), idx);
        by_name
            .entry(simple_name(&f.path).to_string())
            .or_default()
            .push(idx);
    }

    let mut edges: Vec<Edge> = Vec::new();
    let (mut resolved, mut ambiguous, mut unresolved_calls) = (0u32, 0u32, 0u32);
    let empty_use_map = HashMap::new();

    for caller in &functions {
        let Some(body) = caller.body else { continue };
        let mut collector = CallCollector::default();
        collector.visit_block(body);
        let use_map = use_imports.get(&caller.file).unwrap_or(&empty_use_map);

        for call in collector.calls {
            match call {
                RawCall::Path(segments) => {
                    let matches = resolve_call(
                        &segments,
                        &caller.module_path,
                        caller.self_type_bare.as_deref(),
                        &crate_name,
                        use_map,
                        &by_full_path,
                        &by_name,
                    );
                    push_edge(
                        &mut edges,
                        &caller.path,
                        matches,
                        &functions,
                        segments.join("::"),
                        &mut resolved,
                        &mut ambiguous,
                        &mut unresolved_calls,
                    );
                }
                RawCall::Method(name) => {
                    let matches = by_name.get(&name).cloned().unwrap_or_default();
                    push_edge(
                        &mut edges,
                        &caller.path,
                        matches,
                        &functions,
                        name,
                        &mut resolved,
                        &mut ambiguous,
                        &mut unresolved_calls,
                    );
                }
            }
        }
    }

    eprintln!(
        "call graph: {} functions, {resolved} calls resolved, {ambiguous} ambiguous, {unresolved_calls} unresolved (external/std or unrecognized)",
        functions.len()
    );

    if let Some(dot_path) = dot_path
        && let Err(err) = write_dot(&functions, &edges, Path::new(dot_path))
    {
        eprintln!("warning: failed to write dot file {dot_path}: {err}");
    }

    let nodes: Vec<Node> = functions
        .iter()
        .map(|f| Node {
            path: f.path.clone(),
            kind: f.kind.to_string(),
            file: f.file.clone(),
            assurance: f.code_level.as_ref().map(|c| c.assurance.clone()),
            assurance_rank: f.code_level.as_ref().map(|c| c.assurance_rank),
            importance: f.code_level.as_ref().map(|c| c.importance.clone()),
            importance_rank: f.code_level.as_ref().map(|c| c.importance_rank),
        })
        .collect();

    let output = Output {
        annotations: results,
        call_graph: CallGraph { nodes, edges },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
