// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock Binary Format (TBF) header parsing library.

// Parsing the headers does not require any unsafe operations.
#![forbid(unsafe_code)]
#![no_std]
#![cfg_attr(target_feature = "xcheri", feature(int_roundings))]

pub mod parse;
#[allow(dead_code)] // Some fields not read on device, but read when creating headers
pub mod types;
