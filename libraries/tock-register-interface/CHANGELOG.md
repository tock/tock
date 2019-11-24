# Changelog

## master

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
