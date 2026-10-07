// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Definition of Deferred Call tasks.
//!
//! Deferred calls also peripheral drivers to register pseudo interrupts.
//! These are the definitions of which deferred calls this chip needs.

/// A type of task to defer a call for
#[derive(Copy, Clone)]
pub enum DeferredCallTask {
    DateTimeGet = 0,
    DateTimeSet = 1,
}

impl TryFrom<usize> for DeferredCallTask {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::DateTimeGet),
            1 => Ok(Self::DateTimeSet),
            _ => Err(()),
        }
    }
}

impl From<DeferredCallTask> for usize {
    fn from(val: DeferredCallTask) -> Self {
        val as Self
    }
}
