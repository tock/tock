// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Resources for when a board panics.

use kernel::platform::chip::Chip;
use kernel::process::ProcessPrinter;
use kernel::process::ProcessSlot;
use kernel::utilities::cells::MapCell;

/// Typical resources needed by panic handlers when a board panics.
pub struct BoardPanic<'a, C: Chip, PP: ProcessPrinter> {
    /// The array of process slots.
    processes: MapCell<&'static [ProcessSlot]>,
    /// The board-specific chip object.
    chip: MapCell<&'a C>,
    /// The tool for printing process details.
    printer: MapCell<&'a PP>,
}

impl<'a, C: Chip, PP: ProcessPrinter> BoardPanic<'a, C, PP> {
    /// Create a new [`BoardPanic`] with nothing stored.
    pub const fn new() -> Self {
        Self {
            processes: MapCell::empty(),
            chip: MapCell::empty(),
            printer: MapCell::empty(),
        }
    }

    /// Set the process slot array.
    pub fn set_processes(&self, processes: &'static [ProcessSlot]) {
        self.processes.put(processes);
    }

    /// Set the chip reference.
    pub fn set_chip(&self, chip: &'a C) {
        self.chip.put(chip);
    }

    /// Set the process printer reference.
    pub fn set_process_printer(&self, printer: &'a PP) {
        self.printer.put(printer);
    }

    /// Unconditionally get the process slot array.
    pub fn get_processes(&self) -> &'static [ProcessSlot] {
        self.processes.take().unwrap()
    }

    /// Get the chip.
    pub fn get_chip(&self) -> Option<&'a C> {
        self.chip.take()
    }

    /// Get the process printer.
    pub fn get_process_printer(&self) -> Option<&'a PP> {
        self.printer.take()
    }
}
