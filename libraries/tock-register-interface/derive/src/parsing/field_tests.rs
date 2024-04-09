// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::parsing::field::ParsedField;
use crate::parsing::{
    assert_next_contains, MULTIPLE_SAME_OP, NOT_AN_OFFSET, NOT_A_DATA_TYPE, NOT_A_NAME,
    OP_LONG_NAME_SINGLE_OP, SHARED_AND_OP_LONG_NAME, UNKNOWN_ATTRIBUTE, UNKNOWN_OP,
};
use crate::{Aliased, Field, FieldContents, LongNames, Register, Safety};
use pretty_assertions::assert_eq;
use quote::quote;
use syn::{parse2, parse_quote};

#[test]
fn aliased() {
    let ParsedField(result) = parse_quote! {
        _ => ctrl: u8 { Read(Ctrl1), Write(Ctrl2) }
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![],
            comments: vec![],
            contents: FieldContents::Register(Register {
                data_type: parse_quote![u8],
                long_names: LongNames::Aliased(Aliased {
                    read: parse_quote![Ctrl1],
                    write: parse_quote![Ctrl2]
                }),
                name: parse_quote![ctrl],
                read: Some(Safety::Safe(parse_quote![Read])),
                write: Some(Safety::Safe(parse_quote![Write])),
            }),
            offset: None,
        }
    );
}

#[test]
fn aliased_unsafe_read() {
    let ParsedField(result) = parse_quote! {
        _ => ctrl: u8 { UnsafeRead, Write(Ctrl) }
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![],
            comments: vec![],
            contents: FieldContents::Register(Register {
                data_type: parse_quote![u8],
                long_names: LongNames::Aliased(Aliased {
                    read: parse_quote![()],
                    write: parse_quote![Ctrl]
                }),
                name: parse_quote![ctrl],
                read: Some(Safety::Unsafe(parse_quote![UnsafeRead])),
                write: Some(Safety::Safe(parse_quote![Write])),
            }),
            offset: None,
        }
    );
}

#[test]
fn aliased_unsafe_write() {
    let ParsedField(result) = parse_quote! {
        _ => ctrl: u8 { Read(Ctrl), UnsafeWrite }
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![],
            comments: vec![],
            contents: FieldContents::Register(Register {
                data_type: parse_quote![u8],
                long_names: LongNames::Aliased(Aliased {
                    read: parse_quote![Ctrl],
                    write: parse_quote![()]
                }),
                name: parse_quote![ctrl],
                read: Some(Safety::Safe(parse_quote![Read])),
                write: Some(Safety::Unsafe(parse_quote![UnsafeWrite])),
            }),
            offset: None,
        }
    );
}

#[test]
fn bad_attrs_and_bad_offset() {
    let iter = &mut parse2::<ParsedField>(quote! {
        #[derive(UnknownAttr)]
        #[unknown_attr = "3"]
        #[unknown_attr]
        not_an_offset => _
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, NOT_AN_OFFSET);
    assert!(iter.next().is_none());
}

#[test]
fn bad_arrow() {
    // Add a bad attr to confirm that errors prior to the bad => are
    // included.
    let iter = &mut parse2::<ParsedField>(quote! {
        #[unknown_attr]
        _ -> _
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, "=>");
    assert!(iter.next().is_none());
}

#[test]
fn bad_colon() {
    // Add a bad attr to confirm that errors prior to the bad : included.
    let iter = &mut parse2::<ParsedField>(quote! {
        #[unknown_attr]
        _ => a
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, ":");
    assert!(iter.next().is_none());
}

#[test]
fn bad_data_type() {
    // Add a bad attr to confirm that prior errors are included.
    let iter = &mut parse2::<ParsedField>(quote! {
        #[unknown_attr]
        _ => a: 123 {}
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, NOT_A_DATA_TYPE);
    assert!(iter.next().is_none());
}

#[test]
fn bad_name() {
    // Add a bad attr to confirm that errors prior to the bad name are
    // included.
    let iter = &mut parse2::<ParsedField>(quote! {
        #[unknown_attr]
        _ => 123: u32 {}
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, NOT_A_NAME);
    assert!(iter.next().is_none());
}

#[test]
fn bad_shared_long_name() {
    // Add a bad attr to confirm that prior errors are included.
    let iter = &mut parse2::<ParsedField>(quote! {
        #[unknown_attr]
        _ => a: u32(1=2) {}
    })
    .expect_err("parsing should have failed")
    .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, NOT_A_DATA_TYPE);
    assert!(iter.next().is_none());
}

#[test]
fn many_errors_register() {
    let ParsedField(result) = parse_quote! {
        #[msg = "unknown attribute 1"]
        #[unknown_attr_2]
        _ => ctrl: u32(Ctrl) { Read(Ctrl), UnknownOp }
    };
    let iter = &mut result.expect_err("parsing should have failed").into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, UNKNOWN_OP);
    assert_next_contains(iter, OP_LONG_NAME_SINGLE_OP);
    assert!(iter.next().is_none());
}

#[test]
fn no_long_name() {
    let ParsedField(result) = parse_quote! {
        _ => ctrl: u8 { Read, Write }
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![],
            comments: vec![],
            contents: FieldContents::Register(Register {
                data_type: parse_quote![u8],
                long_names: LongNames::Single(parse_quote![()]),
                name: parse_quote![ctrl],
                read: Some(Safety::Safe(parse_quote![Read])),
                write: Some(Safety::Safe(parse_quote![Write])),
            }),
            offset: None,
        }
    );
}

#[test]
fn op_long_name_single_op() {
    let ParsedField(result) = parse_quote![_ => ctrl: u32 { Read(Ctrl) }];
    let iter = &mut result.expect_err("no errors reported").into_iter();
    assert_next_contains(iter, OP_LONG_NAME_SINGLE_OP);
    assert!(iter.next().is_none());
}

#[test]
fn padding() {
    let ParsedField(result) = parse_quote! {
        /// Doc comment 1
        #[cfg(feature = "a")]
        /// Doc comment 2
        #[cfg(not(feature = "b"))]
        0x7 => _
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![
                parse_quote![#[cfg(feature = "a")]],
                parse_quote![#[cfg(not(feature = "b"))]]
            ],
            comments: vec![
                parse_quote![#[doc = r" Doc comment 1"]],
                parse_quote![#[doc = r" Doc comment 2"]]
            ],
            contents: FieldContents::Padding(parse_quote![_]),
            offset: Some(parse_quote![0x7]),
        }
    );
}

#[test]
fn register() {
    let ParsedField(result) = parse_quote! {
        /// Doc comment 1
        #[cfg(feature = "a")]
        /// Doc comment 2
        #[cfg(not(feature = "b"))]
        0x7 => ctrl: [u8; 4](Ctrl) { Read, UnsafeWrite }
    };
    assert_eq!(
        result.expect("parsing failed"),
        Field {
            cfgs: vec![
                parse_quote![#[cfg(feature = "a")]],
                parse_quote![#[cfg(not(feature = "b"))]]
            ],
            comments: vec![
                parse_quote![#[doc = r" Doc comment 1"]],
                parse_quote![#[doc = r" Doc comment 2"]]
            ],
            contents: FieldContents::Register(Register {
                data_type: parse_quote![[u8; 4]],
                long_names: LongNames::Single(parse_quote![Ctrl]),
                name: parse_quote![ctrl],
                read: Some(Safety::Safe(parse_quote![Read])),
                write: Some(Safety::Unsafe(parse_quote![UnsafeWrite])),
            }),
            offset: Some(parse_quote![0x7]),
        }
    );
}

#[test]
fn register_no_ops() {
    let iter = &mut parse2::<ParsedField>(quote![#[unknown_attr] _ => ctrl: u32])
        .expect_err("parsing should have failed")
        .into_iter();
    assert_next_contains(iter, UNKNOWN_ATTRIBUTE);
    assert_next_contains(iter, "expected curly braces");
    assert!(iter.next().is_none());
}

#[test]
fn shared_and_op_long_name() {
    let ParsedField(result) = parse_quote![_ => ctrl: u32(Ctrl) { Read(Ctrl), Write }];
    let iter = &mut result.expect_err("no errors reported").into_iter();
    assert_next_contains(iter, SHARED_AND_OP_LONG_NAME);
    assert!(iter.next().is_none());
}

#[test]
fn unknown_and_duplicate_ops() {
    let ParsedField(result) =
        parse_quote! [_ => ctrl: u32 { Read, UnknownOp1, UnsafeRead, UnknownOp2 }];
    let iter = &mut result.expect_err("no errors reported").into_iter();
    assert_next_contains(iter, UNKNOWN_OP);
    assert_next_contains(iter, MULTIPLE_SAME_OP);
    assert_next_contains(iter, UNKNOWN_OP);
    assert!(iter.next().is_none());
}
