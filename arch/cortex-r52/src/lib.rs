// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

#![no_std]
#![allow(dead_code)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(not(any(feature = "config-xcku115an536")))]
compile_error!("a configuration must be selected");

#[cfg(feature = "config-xcku115an536")]
#[path = "../../../chips/xcku115an536/src/config.rs"]
pub mod config;

pub mod gentimer;
pub mod gic;
pub mod mpu;
pub mod reg;
pub mod support;
pub mod syscall;
pub mod thread_id;
pub mod vectors;
