//! Helper functions related to Tock processes.

use core::convert::TryInto;
use core::fmt;

use crate::capabilities::ProcessManagementCapability;
use crate::config;
use crate::debug;
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;

/// Errors that can occur when trying to load and create processes.
pub enum ProcessLoadError {
    /// The TBF header for the process could not be successfully parsed.
    TbfHeaderParseFailure(tock_tbf::types::TbfParseError),

    /// Not enough flash remaining to parse a process and its header.
    NotEnoughFlash,

    /// Not enough memory to meet the amount requested by a process. Modify the
    /// process to request less memory, flash fewer processes, or increase the
    /// size of the region your board reserves for process memory.
    NotEnoughMemory,

    /// A process was loaded with a length in flash that the MPU does not
    /// support. The fix is probably to correct the process size, but this could
    /// also be caused by a bad MPU implementation.
    MpuInvalidFlashLength,

    /// A process specified a fixed memory address that it needs its memory
    /// range to start at, and the kernel did not or could not give the process
    /// a memory region starting at that address.
    MemoryAddressMismatch {
        actual_address: u32,
        expected_address: u32,
    },

    /// A process specified that its binary must start at a particular address,
    /// and that is not the address the binary is actually placed at.
    IncorrectFlashAddress {
        actual_address: u32,
        expected_address: u32,
    },

    /// Process loading error due (likely) to a bug in the kernel. If you get
    /// this error please open a bug report.
    InternalError,
}

impl From<tock_tbf::types::TbfParseError> for ProcessLoadError {
    /// Convert between a TBF Header parse error and a process load error.
    ///
    /// We note that the process load error is because a TBF header failed to
    /// parse, and just pass through the parse error.
    fn from(error: tock_tbf::types::TbfParseError) -> Self {
        ProcessLoadError::TbfHeaderParseFailure(error)
    }
}

impl fmt::Debug for ProcessLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessLoadError::TbfHeaderParseFailure(tbf_parse_error) => {
                write!(f, "Error parsing TBF header\n")?;
                write!(f, "{:?}", tbf_parse_error)
            }

            ProcessLoadError::NotEnoughFlash => {
                write!(f, "Not enough flash available for app linked list")
            }

            ProcessLoadError::NotEnoughMemory => {
                write!(f, "Not able to meet memory requirements requested by apps")
            }

            ProcessLoadError::MpuInvalidFlashLength => {
                write!(f, "App flash length not supported by MPU")
            }

            ProcessLoadError::MemoryAddressMismatch {
                actual_address,
                expected_address,
            } => write!(
                f,
                "App memory does not match requested address Actual:{:#x}, Expected:{:#x}",
                actual_address, expected_address
            ),

            ProcessLoadError::IncorrectFlashAddress {
                actual_address,
                expected_address,
            } => write!(
                f,
                "App flash does not match requested address. Actual:{:#x}, Expected:{:#x}",
                actual_address, expected_address
            ),

            ProcessLoadError::InternalError => write!(f, "Error in kernel. Likely a bug."),
        }
    }
}

/// Helper function to load processes from flash into an array of active
/// processes. This is the default template for loading processes, but a board
/// is able to create its own `load_processes()` function and use that instead.
///
/// Processes are found in flash starting from the given address and iterating
/// through Tock Binary Format (TBF) headers. Processes are given memory out of
/// the `app_memory` buffer until either the memory is exhausted or the
/// allocated number of processes are created. This buffer is a non-static slice,
/// ensuring that this code cannot hold onto the slice past the end of this function
/// (instead, processes store a pointer and length), which necessary for later
/// creation of `ProcessBuffer`s in this memory region to be sound.
/// A reference to each process is stored in the provided `procs` array.
/// How process faults are handled by the
/// kernel must be provided and is assigned to every created process.
///
/// This function is made `pub` so that board files can use it, but loading
/// processes from slices of flash an memory is fundamentally unsafe. Therefore,
/// we require the `ProcessManagementCapability` to call this function.
///
/// Returns `Ok(())` if process discovery went as expected. Returns a
/// `ProcessLoadError` if something goes wrong during TBF parsing or process
/// creation.
pub fn load_processes<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &mut [u8], // not static, so that process.rs cannot hold on to slice w/o unsafe
    procs: &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
    _capability: &dyn ProcessManagementCapability,
) -> Result<(), ProcessLoadError> {
    if config::CONFIG.debug_load_processes {
        debug!(
            "Loading processes from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X}",
            app_flash.as_ptr() as usize,
            app_flash.as_ptr() as usize + app_flash.len() - 1,
            app_memory.as_ptr() as usize,
            app_memory.as_ptr() as usize + app_memory.len() - 1
        );
    }

    let mut remaining_flash = app_flash;
    let mut remaining_memory = app_memory;

    // Try to discover up to `procs.len()` processes in flash.
    for i in 0..procs.len() {
        // Get the first eight bytes of flash to check if there is another
        // app.
        let test_header_slice = match remaining_flash.get(0..8) {
            Some(s) => s,
            None => {
                // Not enough flash to test for another app. This just means
                // we are at the end of flash, and there are no more apps to
                // load.
                return Ok(());
            }
        };

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        let (version, header_length, entry_length) = match tock_tbf::parse::parse_tbf_header_lengths(
            test_header_slice
                .try_into()
                .or(Err(ProcessLoadError::InternalError))?,
        ) {
            Ok((v, hl, el)) => (v, hl, el),
            Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(entry_length)) => {
                // If we could not parse the header, then we want to skip over
                // this app and look for the next one.
                (0, 0, entry_length)
            }
            Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                // Since Tock apps use a linked list, it is very possible the
                // header we started to parse is intentionally invalid to signal
                // the end of apps. This is ok and just means we have finished
                // loading apps.
                return Ok(());
            }
        };

        // Now we can get a slice which only encompasses the length of flash
        // described by this tbf header.  We will either parse this as an actual
        // app, or skip over this region.
        let entry_flash = remaining_flash
            .get(0..entry_length as usize)
            .ok_or(ProcessLoadError::NotEnoughFlash)?;

        // Advance the flash slice for process discovery beyond this last entry.
        // This will be the start of where we look for a new process since Tock
        // processes are allocated back-to-back in flash.
        remaining_flash = remaining_flash
            .get(entry_flash.len()..)
            .ok_or(ProcessLoadError::NotEnoughFlash)?;

        // Need to reassign remaining_memory in every iteration so the compiler
        // knows it will not be re-borrowed.
        remaining_memory = if header_length > 0 {
            // If we found an actual app header, try to create a `Process`
            // object. We also need to shrink the amount of remaining memory
            // based on whatever is assigned to the new process if one is
            // created.

            // Try to create a process object from that app slice. If we don't
            // get a process and we didn't get a loading error (aka we got to
            // this point), then the app is a disabled process or just padding.
            let (process_option, unused_memory) = unsafe {
                ProcessStandard::create(
                    kernel,
                    chip,
                    entry_flash,
                    header_length as usize,
                    version,
                    remaining_memory,
                    fault_policy,
                    i,
                )?
            };
            process_option.map(|process| {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "Loaded process[{}] from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X} = {:?}",
                        i,
                        entry_flash.as_ptr() as usize,
                        entry_flash.as_ptr() as usize + entry_flash.len() - 1,
                        process.mem_start() as usize,
                        process.mem_end() as usize - 1,
                        process.get_process_name()
                    );
                }

                // Save the reference to this process in the processes array.
                procs[i] = Some(process);
            });
            unused_memory
        } else {
            // We are just skipping over this region of flash, so we have the
            // same amount of process memory to allocate from.
            remaining_memory
        };
    }

    Ok(())
}
