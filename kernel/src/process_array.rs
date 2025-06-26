// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Data structure for storing `Process`es.
//!
//! Many Tock boards store a fixed-length array of process control blocks
//! (PCB) for easy management and traversal of running processes. The
//! `ProcessArray` type facilitates this.
//!
//! The general type for the process array abstraction is
//! `[&Process; NUM_PROCS]`. That is, the array is only sized to store
//! references to each PCB. The actual PCB is allocated in the process's
//! allocated memory.

use crate::process;
use core::cell::Cell;

/// Represents a slot for a process in a [`ProcessArray`].
///
/// A slot can be empty (`None`), or hold a reference to a
/// [`Process`](process::Process).
///
/// The `ProcessSlot` type is useful for allowing slices of processes without
/// knowing the fixed number of processes, or being templated on `NUM_PROCS`.
/// That is, interfaces can use `[ProcessSlot]` to just use an array of process
/// slots.
#[derive(Clone)]
pub struct ProcessSlot {
    /// Optionally points to a process.
    pub(crate) proc: Cell<Option<&'static dyn process::Process>>,
}

impl ProcessSlot {
    pub(crate) fn set(&self, process: &'static dyn process::Process) {
        self.proc.set(Some(process));
    }

    /// Return the underlying [`process::Process`] if the slot contains a
    /// process.
    pub fn get(&self) -> Option<&'static dyn process::Process> {
        self.proc.get()
    }

    /// Check if the slot contains a process with a matching process ID.
    pub fn contains_process_with_id(&self, identifier: usize) -> bool {
        match self.proc.get() {
            Some(process) => process.processid().id() == identifier,
            None => false,
        }
    }
}

/// Storage for a fixed-size array of `Process`es.
pub struct ProcessArray<const NUM_PROCS: usize> {
    processes: [ProcessSlot; NUM_PROCS],
}

impl<const NUM_PROCS: usize> ProcessArray<NUM_PROCS> {
    pub const fn new() -> Self {
        Self {
            processes: [const {
                ProcessSlot {
                    proc: Cell::new(None),
                }
            }; NUM_PROCS],
        }
    }

    pub fn as_slice(&self) -> &[ProcessSlot] {
        &self.processes
    }
}

impl<const NUM_PROCS: usize> core::ops::Index<usize> for ProcessArray<NUM_PROCS> {
    type Output = ProcessSlot;

    fn index(&self, i: usize) -> &ProcessSlot {
        &self.processes[i]
    }
}
