# Changelog

## v0.6

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
