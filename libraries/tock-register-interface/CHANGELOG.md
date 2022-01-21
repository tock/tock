# Changelog

## master

## v0.7

 - #2642: Rename `IntLike` to `UIntLike` to match semantics
 - #2618: Reorganize, document, and feature-gate modules and exports
 - #2589: Upgrade nightly for `const_fn` -> `const_fn_trait_bound`
 - #2517: Use traits for accessing / manipulating registers
 - #2512: Fix `Copy` and `Clone` implementation on `Field`
 - #2300: Add support for `usize`
 - #2220: Remove duplicate code, make local register copy read-write
 - #2210: Add `u128` to `IntLike`
 - #2197: Accept trailing comma in bitfields and bitmasks

## v0.6

 - #2095: Fix syntax errors and inconsistencies in documentation
 - #2071: Clarify bit widths in documentation examples
 - #2015: Use UnsafeCell in registers (see issue #2005)
 - #1939: Make the Field::mask and FieldValue::mask fields private
 - #1823: Allow large unsigned values as bitmasks + add bitmask! helper macro
 - #1554: Allow lifetime parameters for `register_structs! { Foo<'a> { ..`
 - #1661: Add `Aliased` register type for MMIO with differing R/W behavior

## v0.5

 - #1510
   - Register visibility granularity: don't automatically make everything
      `pub`, rather give creation macro callers visbility control.

 - #1489
   - Make `register_structs!` unit test generation opt-out, so that
     `custom-test-frameworks` environments can disable them.

 - #1481
   - Add `#[derive(Copy, Clone)]` to InMemoryRegister.

 - #1428
   - Implement `mask()` for `FieldValue<u16>` which seems to have been
     skipped at some point.
   - Implement `read()` for `FieldValue` so that individual fields
     can be extracted from a register `FieldValue` representation.

 - #1461: Update `register_structs` macro to support flexible visibility of each
   struct and each field. Also revert to private structs by default.

## v0.4.1

 - #1458: Update struct macro to create `pub` structs

## v0.4

 - #1368: Remove `new()` and add `InMemoryRegister`
 - #1410: Add new macro for generating structs

## v0.3

 - #1243: Update to Rust 2018 (nightly)
 - #1250: Doc-only: Fix some rustdoc warnings

## v0.2

 - #1161: Add `read_as_enum` to `LocalRegisterCopy`; thanks @andre-richter

## v0.1 - Initial Release
