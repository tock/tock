// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Tools for displaying process state.

use core::fmt::Write;

use kernel::process::Process;
use kernel::process::{ProcessPrinter, ProcessPrinterContext};
use kernel::utilities::binary_write::BinaryWrite;
use kernel::utilities::binary_write::WriteToBinaryOffsetWrapper;

/// A Process Printer that displays a process as a human-readable string.
pub struct ProcessPrinterText {}

impl ProcessPrinterText {
    pub fn new() -> ProcessPrinterText {
        ProcessPrinterText {}
    }
}

impl ProcessPrinter for ProcessPrinterText {
    // `print_overview()` must be synchronous, but does not assume a synchronous
    // writer or an infinite (or very large) underlying buffer in the writer. To
    // do this, this implementation assumes the underlying writer _is_
    // synchronous. This makes the printing code cleaner, as it does not need to
    // be broken up into chunks of some length (which would need to match the
    // underlying buffer length). However, not all writers are synchronous, so
    // this implementation keeps track of how many bytes were sent on the last
    // call, and only prints new bytes on the next call. This works by having
    // the function start from the beginning each time, formats the entire
    // overview message, and just drops bytes until getting back to where it
    // left off on the last call.
    //
    // ### Assumptions
    //
    // This implementation makes two assumptions:
    // 1. That `print_overview()` is not called in performance-critical code.
    //    Since each time it formats and "prints" the message starting from the
    //    beginning, it duplicates a fair bit of formatting work. Since this is
    //    for debugging, the performance impact of that shouldn't matter.
    // 2. That `printer_overview()` will be called in a tight loop, and no
    //    process state will change between calls. That could change the length
    //    of the printed message, and lead to gaps or parts of the overview
    //    being duplicated. However, it does not make sense that the kernel
    //    would want to run the process while it is displaying debugging
    //    information about it, so this should be a safe assumption.
    fn print_overview(
        &self,
        process: &dyn Process,
        writer: &mut dyn BinaryWrite,
        context: Option<ProcessPrinterContext>,
    ) -> Option<ProcessPrinterContext> {
        let offset = context.map_or(0, |c| c.offset);

        // Process statistics
        let events_queued = process.pending_tasks();
        let syscall_count = process.debug_syscall_count();
        let dropped_upcall_count = process.debug_dropped_upcall_count();
        let restart_count = process.get_restart_count();

        let addresses = process.get_addresses();
        let sizes = process.get_sizes();

        let process_struct_memory_location = addresses.sram_end
            - sizes.grant_pointers
            - sizes.upcall_list
            - sizes.process_control_block;
        let sram_grant_size = process_struct_memory_location - addresses.sram_grant_start;

        let mut bww = WriteToBinaryOffsetWrapper::new(writer);
        bww.set_offset(offset);

        let _ = bww.write_fmt(format_args!(
            "\
                 𝐀𝐩𝐩: {}   -   [{:?}]\
                 \r\n Events Queued: {}   Syscall Count: {}   Dropped Upcall Count: {}\
                 \r\n Restart Count: {}\
                 \r\n",
            process.get_process_name(),
            process.get_state(),
            events_queued,
            syscall_count,
            dropped_upcall_count,
            restart_count,
        ));

        let _ = match process.debug_syscall_last() {
            Some(syscall) => bww.write_fmt(format_args!(" Last Syscall: {:?}\r\n", syscall)),
            None => bww.write_str(" Last Syscall: None\r\n"),
        };

        let _ = match process.get_completion_code() {
            Some(opt_cc) => match opt_cc {
                Some(cc) => bww.write_fmt(format_args!(" Completion Code: {}\r\n", cc as isize)),
                None => bww.write_str(" Completion Code: Faulted\r\n"),
            },
            None => bww.write_str(" Completion Code: None\r\n"),
        };

        let _ = bww.write_fmt(format_args!(
            "\
                 \r\n\
                 \r\n ╔═══════════╤══════════════════════════════════════════╗\
                 \r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
                 \r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
                 \r\n             │ Grant Ptrs   {:6}\
                 \r\n             │ Upcalls      {:6}\
                 \r\n             │ Process      {:6}\
                 \r\n  {:#010X} ┼───────────────────────────────────────────\
                 \r\n             │ ▼ Grant      {:6}\
                 \r\n  {:#010X} ┼───────────────────────────────────────────\
                 \r\n             │ Unused\
                 \r\n  {:#010X} ┼───────────────────────────────────────────",
            addresses.sram_end,
            sizes.grant_pointers,
            sizes.upcall_list,
            sizes.process_control_block,
            process_struct_memory_location,
            sram_grant_size,
            addresses.sram_grant_start,
            addresses.sram_app_brk,
        ));

        // We check to see if the underlying writer has more work to do. If it
        // does, then its buffer is full and any additional writes are just
        // going to be dropped. So, we skip doing more printing if there are
        // bytes remaining as a slight performance optimization.
        if !bww.bytes_remaining() {
            match addresses.sram_heap_start {
                Some(sram_heap_start) => {
                    let sram_heap_size = addresses.sram_app_brk - sram_heap_start;
                    let sram_heap_allocated = addresses.sram_grant_start - sram_heap_start;

                    let _ = bww.write_fmt(format_args!(
                        "\
                         \r\n             │ ▲ Heap       {:6} | {:6}{}     S\
                         \r\n  {:#010X} ┼─────────────────────────────────────────── R",
                        sram_heap_size,
                        sram_heap_allocated,
                        exceeded_check(sram_heap_size, sram_heap_allocated),
                        sram_heap_start,
                    ));
                }
                None => {
                    let _ = bww.write_str(
                        "\
                         \r\n             │ ▲ Heap            ? |      ?               S\
                         \r\n  ?????????? ┼─────────────────────────────────────────── R",
                    );
                }
            }
        }

        if !bww.bytes_remaining() {
            match (addresses.sram_heap_start, addresses.sram_stack_top) {
                (Some(sram_heap_start), Some(sram_stack_top)) => {
                    let sram_data_size = sram_heap_start - sram_stack_top;
                    let sram_data_allocated = sram_data_size;

                    let _ = bww.write_fmt(format_args!(
                        "\
                         \r\n             │ Data         {:6} | {:6}               A",
                        sram_data_size, sram_data_allocated,
                    ));
                }
                _ => {
                    let _ = bww.write_str(
                        "\
                         \r\n             │ Data              ? |      ?               A",
                    );
                }
            }
        }

        if !bww.bytes_remaining() {
            match (addresses.sram_stack_top, addresses.sram_stack_bottom) {
                (Some(sram_stack_top), Some(sram_stack_bottom)) => {
                    let sram_stack_size = sram_stack_top - sram_stack_bottom;
                    let sram_stack_allocated = sram_stack_top - addresses.sram_start;

                    let _ = bww.write_fmt(format_args!(
                        "\
                         \r\n  {:#010X} ┼─────────────────────────────────────────── M\
                         \r\n             │ ▼ Stack      {:6} | {:6}{}",
                        sram_stack_top,
                        sram_stack_size,
                        sram_stack_allocated,
                        exceeded_check(sram_stack_size, sram_stack_allocated),
                    ));
                }
                _ => {
                    let _ = bww.write_str(
                        "\
                         \r\n  ?????????? ┼─────────────────────────────────────────── M\
                         \r\n             │ ▼ Stack           ? |      ?",
                    );
                }
            }
        }

        if !bww.bytes_remaining() {
            let flash_protected_size = addresses.flash_non_protected_start - addresses.flash_start;
            let flash_app_size = addresses.flash_end - addresses.flash_non_protected_start;

            let _ = bww.write_fmt(format_args!(
                "\
                 \r\n  {:#010X} ┼───────────────────────────────────────────\
                 \r\n             │ Unused\
                 \r\n  {:#010X} ┴───────────────────────────────────────────\
                 \r\n             .....\
                 \r\n  {:#010X} ┬─────────────────────────────────────────── F\
                 \r\n             │ App Flash    {:6}                        L\
                 \r\n  {:#010X} ┼─────────────────────────────────────────── A\
                 \r\n             │ Protected    {:6}                        S\
                 \r\n  {:#010X} ┴─────────────────────────────────────────── H\
                 \r\n",
                addresses.sram_stack_bottom.unwrap_or(0),
                addresses.sram_start,
                addresses.flash_end,
                flash_app_size,
                addresses.flash_non_protected_start,
                flash_protected_size,
                addresses.flash_start
            ));
        }

        if bww.bytes_remaining() {
            // The underlying writer is indicating there are still bytes
            // remaining to be sent. That means we want to return a context so
            // the caller knows to call us again and we can keep printing until
            // we have displayed the entire process overview.
            let new_context = ProcessPrinterContext {
                offset: bww.get_index(),
            };
            Some(new_context)
        } else {
            None
        }
    }
}

/// If `size` is greater than `allocated` then it returns a warning string to
/// help with debugging.
fn exceeded_check(size: usize, allocated: usize) -> &'static str {
    if size > allocated {
        " EXCEEDED!"
    } else {
        "          "
    }
}
