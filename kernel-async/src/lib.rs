// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

#![no_std]
#![feature(impl_trait_in_assoc_type)]
// Tock does not use threads so Futures
// do not have to be Send
#![allow(clippy::future_not_send)]

pub mod delay;
pub mod examples;
pub mod executor;
