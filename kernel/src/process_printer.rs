// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tools for displaying process state.

use crate::process::Process;
use crate::utilities::binary_write::BinaryWrite;

/// A context token that the caller must pass back to us. This allows us to
/// track where we are in the print operation.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ProcessPrinterContext {
    /// The overall print message is broken in to chunks so that it can be fit
    /// in a small buffer that is called multiple times. This tracks which byte
    /// we are at so we can ignore the text before and print the next bytes.
    pub offset: usize,
}

/// Trait for creating a custom "process printer" that formats process state in
/// some sort of presentable format.
///
/// Typically, implementations will display process state in a text UI over some
/// sort of terminal.
///
/// This trait also allows for experimenting with different process display
/// formats. For example, some use cases might want more or less detail, or to
/// encode the process state in some sort of binary format that can be expanded
/// into a human readable format later. Other cases might want to log process
/// state to nonvolatile storage rather than display it immediately.
pub trait ProcessPrinter {
    /// Print a process overview to the `writer`. As `print_overview()` uses a
    /// `&dyn Process` to access the process, only state which can be accessed
    /// via the `Process` trait can be printed.
    ///
    /// This is a synchronous function which also supports asynchronous
    /// operation. This function does not issue a callback, but the return value
    /// indicates whether the caller should call `print_overview()` again (after
    /// the underlying write operation finishes). This allows asynchronous
    /// implementations to still use `print_overview()`, while still supporting
    /// the panic handler which runs synchronously.
    ///
    /// When `print_overview()` is called the first time `None` should be passed
    /// in for `context`.
    ///
    /// ### Return Value
    ///
    /// The return indicates whether `print_overview()` has more printing to do
    /// and should be called again. If `print_overview()` returns `Some()` then
    /// the caller should call `print_overview()` again (providing the returned
    /// `ProcessPrinterContext` as the `context` argument) once the `writer` is
    /// ready to accept more data. If `print_overview()` returns `None`, the
    /// `writer` indicated it accepted all output and the caller does not need
    /// to call `print_overview()` again to finish the printing.
    fn print_overview(
        &self,
        process: &dyn Process,
        writer: &mut dyn BinaryWrite,
        context: Option<ProcessPrinterContext>,
    ) -> Option<ProcessPrinterContext>;
}
