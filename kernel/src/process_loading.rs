// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Reading process binaries and loading them into in-memory Tock processes.
//!
//! The process loader is responsible for parsing the binary formats of Tock processes,
//! checking whether they are allowed to be loaded, and if so initializing a process
//! structure to run it.

use core::convert::TryInto;
use core::fmt;

use crate::capabilities::{ProcessApprovalCapability, ProcessManagementCapability};
use crate::config;
use crate::create_capability;
use crate::debug;
use crate::kernel::{Kernel, ProcessCheckerMachine};
use crate::platform::chip::Chip;
use crate::platform::platform::KernelResources;
use crate::process::{Process, ShortID};
use crate::process_checker::AppCredentialsChecker;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;

/// Errors that can occur when trying to load and create processes.
pub enum ProcessLoadError {
    /// No TBF header was found.
    TbfHeaderNotFound,

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

    /// The MPU configuration failed for some other, unspecified reason. This
    /// could be of an internal resource exhaustion, or a mismatch between the
    /// (current) MPU constraints and process requirements.
    MpuConfigurationError,

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

    /// A process requires a newer version of the kernel or did not specify
    /// a required version. Processes can include the KernelVersion TBF header stating
    /// their compatible kernel version (^major.minor).
    ///
    /// Boards may not require processes to include the KernelVersion TBF header, and
    /// the kernel supports ignoring a missing KernelVersion TBF header. In that case,
    /// this error will not be returned for a process missing a KernelVersion TBF
    /// header.
    ///
    /// `version` is the `(major, minor)` kernel version the process indicates it
    /// requires. If `version` is `None` then the process did not include the
    /// KernelVersion TBF header.
    IncompatibleKernelVersion { version: Option<(u16, u16)> },

    /// The application checker requires credentials, but the TBF did
    /// not include a credentials that meets the checker's
    /// requirements. This can be either because the TBF has no
    /// credentials or the checker policy did not accept any of the
    /// credentials it has.
    CredentialsNoAccept,

    /// The process contained a credentials which was rejected by the verifier.
    /// The u32 indicates which credentials was rejected: the first credentials
    /// after the application binary is 0, and each subsequent credentials increments
    /// this counter.
    CredentialsReject(u32),

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
            ProcessLoadError::TbfHeaderNotFound => {
                write!(f, "Could not find TBF header")
            }

            ProcessLoadError::TbfHeaderParseFailure(tbf_parse_error) => {
                writeln!(f, "Error parsing TBF header")?;
                write!(f, "{:?}", tbf_parse_error)
            }

            ProcessLoadError::NotEnoughFlash => {
                write!(f, "Not enough flash available for TBF")
            }

            ProcessLoadError::NotEnoughMemory => {
                write!(f, "Not able to provide RAM requested by app")
            }

            ProcessLoadError::MpuInvalidFlashLength => {
                write!(f, "App flash length not supported by MPU")
            }

            ProcessLoadError::MpuConfigurationError => {
                write!(f, "Configuring the MPU failed")
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

            ProcessLoadError::IncompatibleKernelVersion { version } => match version {
                Some((major, minor)) => write!(
                    f,
                    "Process is incompatible with the kernel. Running: {}.{}, Requested: {}.{}",
                    crate::KERNEL_MAJOR_VERSION,
                    crate::KERNEL_MINOR_VERSION,
                    major,
                    minor
                ),
                None => write!(f, "Process did not provide a TBF kernel version header"),
            },

            ProcessLoadError::CredentialsNoAccept => write!(f, "No credentials accepted."),

            ProcessLoadError::CredentialsReject(index) => {
                write!(f, "Credentials index {} rejected.", index)
            }

            ProcessLoadError::InternalError => write!(f, "Error in kernel. Likely a bug."),
        }
    }
}

/// Load processes (stored as TBF objects in flash) into runnable
/// process structures stored in the `procs` array. If the kernel is
/// configured with an `AppCredentialsChecker`, this method scans the
/// footers in the TBF for cryptographic credentials for binary
/// integrity, passing them to the checker to decide whether the
/// process has sufficient credentials to run.
///
/// This function is made `pub` so that board files can use it, but loading
/// processes from slices of flash an memory is fundamentally unsafe. Therefore,
/// we require the `ProcessManagementCapability` to call this function.
// Mark inline always to reduce code size. Since this is only called in one
// place (a board's main.rs), by inlining the load_*processes() functions, the
// compiler can elide many checks which reduces code size appreciably. Note,
// however, these functions require a rather large stack frame, which may be an
// issue for boards small kernel stacks.
#[inline(always)]
pub fn load_and_check_processes<KR: KernelResources<C>, C: Chip>(
    kernel: &'static Kernel,
    kernel_resources: &KR,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8],
    mut procs: &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
    _capability_management: &dyn ProcessManagementCapability,
) -> Result<(), ProcessLoadError>
where
    <KR as KernelResources<C>>::CredentialsCheckingPolicy: 'static,
{
    load_processes_from_flash(
        kernel,
        chip,
        app_flash,
        app_memory,
        &mut procs,
        fault_policy,
    )?;
    let _res = check_processes(kernel_resources, kernel.get_checker());
    Ok(())
}

/// Load processes (stored as TBF objects in flash) into runnable
/// process structures stored in the `procs` array and mark all
/// successfully loaded processes as runnable. This method does not
/// check the cryptographic credentials of TBF objects. Platforms
/// for which code size is tight and do not need to check TBF
/// credentials can call this method instead of `load_and_check_processes`
/// because it results in a smaller kernel, as it does not invoke
/// the credential checking state machine.
///
/// This function is made `pub` so that board files can use it, but loading
/// processes from slices of flash an memory is fundamentally unsafe. Therefore,
/// we require the `ProcessManagementCapability` to call this function.
// Mark inline always to reduce code size. Since this is only called in one
// place (a board's main.rs), by inlining the load_*processes() functions, the
// compiler can elide many checks which reduces code size appreciably. Note,
// however, these functions require a rather large stack frame, which may be an
// issue for boards small kernel stacks.
#[inline(always)]
pub fn load_processes<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8],
    mut procs: &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
    _capability_management: &dyn ProcessManagementCapability,
) -> Result<(), ProcessLoadError> {
    load_processes_from_flash(
        kernel,
        chip,
        app_flash,
        app_memory,
        &mut procs,
        fault_policy,
    )?;

    if config::CONFIG.debug_process_credentials {
        debug!("Checking: no checking, load and run all processes");
    }
    let capability = create_capability!(ProcessApprovalCapability);
    for proc in procs.iter() {
        let res = proc.map(|p| {
            p.mark_credentials_pass(None, ShortID::LocallyUnique, &capability)
                .or(Err(ProcessLoadError::InternalError))?;
            if config::CONFIG.debug_process_credentials {
                debug!("Running {}", p.get_process_name());
            }
            Ok(())
        });
        if let Some(Err(e)) = res {
            return Err(e);
        }
    }
    Ok(())
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
/// Returns `Ok(())` if process discovery went as expected. Returns a
/// `ProcessLoadError` if something goes wrong during TBF parsing or process
/// creation.
#[inline(always)]
fn load_processes_from_flash<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8],
    procs: &mut &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
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
    let mut index = 0;
    let num_procs = procs.len();
    while index < num_procs {
        let load_result = load_process(
            kernel,
            chip,
            remaining_flash,
            remaining_memory,
            index,
            fault_policy,
        );
        match load_result {
            Ok((new_flash, new_mem, proc)) => {
                remaining_flash = new_flash;
                remaining_memory = new_mem;
                if proc.is_some() {
                    if config::CONFIG.debug_load_processes {
                        proc.map(|p| debug!("Loaded process {}", p.get_process_name()));
                    }
                    procs[index] = proc;
                    index += 1;
                } else {
                    if config::CONFIG.debug_load_processes {
                        debug!("No process loaded.");
                    }
                }
            }
            Err((_new_flash, _new_mem, err)) => {
                if config::CONFIG.debug_load_processes {
                    debug!("No more processes to load: {:?}.", err);
                }
                // No more processes to load.
                break;
            }
        }
    }
    Ok(())
}

/// Use `checker` to transition `procs` from the
/// `CredentialsUnchecked` state into the `CredentialsApproved` state
/// (if they pass the checker policy) or `CredentialsFailed` state (if
/// they do not pass the checker policy). When the kernel encounters a
/// process in the `CredentialsApproved` state, it starts the process
/// by enqueueing a stack frame to run the initialization function as
/// indicated in the TBF header.
#[inline(always)]
fn check_processes<KR: KernelResources<C>, C: Chip>(
    kernel_resources: &KR,
    machine: &'static ProcessCheckerMachine,
) -> Result<(), ProcessLoadError> {
    let policy = kernel_resources.credentials_checking_policy();
    machine.set_policy(policy);
    policy.set_client(machine);
    machine.next()?;
    Ok(())
}

/// Load a process stored as a TBF process binary at the start of `app_flash`,
/// with `app_memory` as the RAM pool that its RAM should be allocated from.
/// Returns `Ok` if there are possibly more processes and `load_process` should
/// be called again, `Err` if it should not be. May return `Ok` with `None` if
/// a process was not found (e..g, there was padding) but there may be more
/// processes.
fn load_process<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8],
    index: usize,
    fault_policy: &'static dyn ProcessFaultPolicy,
) -> Result<
    (
        &'static [u8],
        &'static mut [u8],
        Option<&'static dyn Process>,
    ),
    (&'static [u8], &'static mut [u8], ProcessLoadError),
> {
    if config::CONFIG.debug_load_processes {
        debug!(
            "Loading process from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X}",
            app_flash.as_ptr() as usize,
            app_flash.as_ptr() as usize + app_flash.len() - 1,
            app_memory.as_ptr() as usize,
            app_memory.as_ptr() as usize + app_memory.len() - 1
        );
    }
    let test_header_slice = match app_flash.get(0..8) {
        Some(s) => s,
        None => {
            // Not enough flash to test for another app. This just means
            // we are at the end of flash, and there are no more apps to
            // load.
            return Err((app_flash, app_memory, ProcessLoadError::NotEnoughFlash));
        }
    };

    // Pass the first eight bytes to tbfheader to parse out the length of
    // the tbf header and app. We then use those values to see if we have
    // enough flash remaining to parse the remainder of the header.
    let header = test_header_slice.try_into();
    if header.is_err() {
        return Err((app_flash, app_memory, ProcessLoadError::InternalError));
    }
    let header = header.unwrap();

    let (version, header_length, entry_length) =
        match tock_tbf::parse::parse_tbf_header_lengths(header) {
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
                return Err((app_flash, app_memory, ProcessLoadError::TbfHeaderNotFound));
            }
        };

    // Now we can get a slice which only encompasses the length of flash
    // described by this tbf header.  We will either parse this as an actual
    // app, or skip over this region.
    let entry_flash = match app_flash.get(0..entry_length as usize) {
        None => return Err((app_flash, app_memory, ProcessLoadError::NotEnoughFlash)),
        Some(val) => val,
    };

    // Advance the flash slice for process discovery beyond this last entry.
    // This will be the start of where we look for a new process since Tock
    // processes are allocated back-to-back in flash.
    let remaining_flash = match app_flash.get(entry_flash.len()..) {
        None => return Err((app_flash, app_memory, ProcessLoadError::NotEnoughFlash)),
        Some(val) => val,
    };

    // Need to reassign remaining_memory in every iteration so the compiler
    // knows it will not be re-borrowed.
    let (process_option, remaining_memory) = if header_length > 0 {
        // If we found an actual app header, try to create a `Process`
        // object. We also need to shrink the amount of remaining memory
        // based on whatever is assigned to the new process if one is
        // created.

        // Try to create a process object from that app slice. If we don't
        // get a process and we didn't get a loading error (aka we got to
        // this point), then the app is a disabled process or just padding.
        let (process_option, unused_memory) = unsafe {
            let result = ProcessStandard::create(
                kernel,
                chip,
                entry_flash,
                header_length as usize,
                version,
                app_memory,
                fault_policy,
                true,
                index,
            );
            match result {
                Ok(tuple) => tuple,
                Err((err, memory)) => {
                    return Err((remaining_flash, memory, err));
                }
            }
        };
        process_option.map(|process| {
            if config::CONFIG.debug_load_processes {
                debug!(
                    "Loaded process[{}] from flash={:#010X}-{:#010X} into sram={:#010X}-{:#010X} = {:?}",
                    index,
                    entry_flash.as_ptr() as usize,
                    entry_flash.as_ptr() as usize + entry_flash.len() - 1,
                    process.get_addresses().sram_start ,
                    process.get_addresses().sram_end  - 1,
                    process.get_process_name()
                );
            }
        });
        (process_option, unused_memory)
    } else {
        // We are just skipping over this region of flash, so we have the
        // same amount of process memory to allocate from.
        (None, app_memory)
    };
    Ok((remaining_flash, remaining_memory, process_option))
}
