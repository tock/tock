unsafe-requirements
===================

Scans a Rust crate (or workspace) for `unsafe { ... }` blocks, works out which
specific operations inside each block are the reason it has to be `unsafe`,
and annotates the block with comment lines linking to the Rust documentation
that explains each one. For example, running with `--write` turns this:

```rust
pub fn read_register(ptr: *const u32) -> u32 {
    unsafe { *ptr }
}
```

into this:

```rust
pub fn read_register(ptr: *const u32) -> u32 {
    //
    // unsafe-requirements: dereferencing a raw pointer: https://doc.rust-lang.org/reference/unsafety.html#r-safety.unsafe-deref
    unsafe { *ptr }
}
```

The goal is to give a reviewer (or the author) a quick link to *why* Rust
requires `unsafe` here, in addition to — not instead of — a human-written
`// SAFETY: ...` comment explaining why the operation is actually sound in
context.

A call into an unsafe function or method defined somewhere in the scanned
source (rather than in `core`/`alloc`) has no external documentation to link
to, so it's labeled directly instead:

```rust
unsafe fn disable_mpu(mpu: &dyn Mpu) { /* ... */ }

pub fn panic(mpu: &dyn Mpu) {
    //
    // unsafe-requirements: calling unsafe Tock function `disable_mpu`
    unsafe { disable_mpu(mpu) }
}
```

## Usage

```shell
# Dry run: print every unsafe block found and the reasons for each,
# without changing any files.
cargo run -p unsafe-requirements -- path/to/crate

# Insert/update the `// unsafe-requirements: ...` comments in place.
cargo run -p unsafe-requirements -- --write path/to/crate
```

With no path, the current directory is scanned. The tool walks all `*.rs`
files under the given path (respecting `.gitignore`), so it can be pointed at
a single crate or at an entire workspace such as `kernel/` or `arch/`.

Re-running with `--write` is idempotent: a block's comment is regenerated
(and removed entirely if no operation is recognized anymore), rather than
duplicated, so it's safe to run repeatedly as code changes.

## How it works

1. Every `*.rs` file under the given path is parsed (using [`syn`][syn]) and
   scanned once to build a crate-wide inventory of the names of `unsafe fn`s
   and methods, `static mut` variables, and `union` fields — wherever in the
   crate they're declared.
2. Each file is then scanned again for `unsafe { ... }` blocks. For every
   block, its contents (but not the contents of any block nested inside it —
   that one is responsible for its own operations) are inspected for:
   - dereferencing a raw pointer (`*ptr`)
   - calling an unsafe function or method, either one declared somewhere in
     the scanned source (found in step 1) or one of a curated list of
     well-known unsafe functions/methods in `core`/`alloc`
     (`core::mem::transmute`, `<*const T>::add`, `<[T]>::get_unchecked`, ...)
   - reading or writing a `static mut` variable
   - accessing a `union` field
   - inline assembly (`asm!`/`naked_asm!`)
3. Each recognized operation is mapped to a URL (see `src/urls.rs`) — mostly
   anchors into the ["Unsafety"][unsafety] chapter of the Rust Reference, plus
   direct links to the relevant `core`/`alloc` docs for specific well-known
   functions — and, with `--write`, one `// unsafe-requirements: ...` comment
   line per operation is inserted directly above the block. The one exception
   is a call into an unsafe function/method declared in the scanned source
   itself: there's no external documentation to point at, so it's just
   labeled "calling unsafe Tock function/method `name`" with no URL.

[syn]: https://docs.rs/syn/
[unsafety]: https://doc.rust-lang.org/reference/unsafety.html

## Limitations

This is a **syntactic, best-effort** tool: it does not run the Rust type
checker (`rustc`/`cargo check`), so it cannot always tell what an expression's
type is. In particular:

- A raw pointer dereference (`*ptr`) is detected purely by syntax, so a block
  that only dereferences a reference or smart pointer for an unrelated reason
  would not normally appear here — but if it does appear inside an `unsafe`
  block for some other reason, it's reported as if it were a raw pointer
  dereference.
- A `union` field access is recognized only by field *name*, matched against
  every union field declared anywhere in the crate — it does not verify that
  the accessed value is actually of that union's type.
- Calls to well-known `core`/`alloc` unsafe functions are recognized by their
  final name only (e.g. `add`, `new_unchecked`), which can occasionally
  collide with an unrelated safe function or method of the same name. The
  same is true, with the same caveat, for calls into unsafe functions/methods
  declared in the scanned source itself.
- Some unsafe operations have no generic syntactic signature at all and are
  not detected: for example, a safety requirement coming from inside a
  project-specific macro invocation, or calling a safe function annotated
  with `#[target_feature]`.

When a block contains none of the recognized operations, it is left alone
(no comment is inserted) and reported in the dry-run output as needing manual
review — this happens for blocks whose safety requirement doesn't come from
one of the operations above (for instance, a safety invariant upheld only by
the surrounding code, not by any single unsafe operation), or one this tool
doesn't yet recognize.

This tool is a documentation aid, not a safety verifier: it says nothing
about whether a given `unsafe` block is actually *sound*, only about which
Rust-language rule requires it to be written as `unsafe` in the first place.

## Tests

`cargo test -p unsafe-requirements` runs unit tests against the fixture in
`testdata/basic.rs`, which exercises each recognized operation category plus
the nested-block and idempotent-rewrite behavior.
