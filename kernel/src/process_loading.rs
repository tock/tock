// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Helper functions and machines for loading process binaries into in-memory
//! Tock processes.
//!
//! Process loaders are responsible for parsing the binary formats of Tock
//! processes, checking whether they are allowed to be loaded, and if so
//! initializing a process structure to run it.
//!
//! This module provides multiple process loader options depending on which
//! features a particular board requires.

use core::cell::Cell;
use core::fmt;

use crate::capabilities::ProcessManagementCapability;
use crate::config;
use crate::debug;
use crate::deferred_call::{DeferredCall, DeferredCallClient};
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::{Process, ShortId};
use crate::process_binary::{ProcessBinary, ProcessBinaryError};
use crate::process_checker::{AppIdPolicy, ProcessCheckError, ProcessCheckerMachine};
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;
use crate::utilities::cells::{MapCell, OptionalCell};

/// Errors that can occur when trying to load and create processes.
pub enum ProcessLoadError {
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

    /// There is nowhere in the `PROCESSES` array to store this process.
    NoProcessSlot,

    /// Process loading failed because parsing the binary failed.
    BinaryError(ProcessBinaryError),

    /// Process loading failed because checking the process failed.
    CheckError(ProcessCheckError),

    /// Process loading error due (likely) to a bug in the kernel. If you get
    /// this error please open a bug report.
    InternalError,
}

impl fmt::Debug for ProcessLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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

            ProcessLoadError::NoProcessSlot => {
                write!(f, "Nowhere to store the loaded process")
            }

            ProcessLoadError::BinaryError(binary_error) => {
                writeln!(f, "Error parsing process binary")?;
                write!(f, "{:?}", binary_error)
            }

            ProcessLoadError::CheckError(check_error) => {
                writeln!(f, "Error checking process")?;
                write!(f, "{:?}", check_error)
            }

            ProcessLoadError::InternalError => write!(f, "Error in kernel. Likely a bug."),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// SYNCHRONOUS PROCESS LOADING
////////////////////////////////////////////////////////////////////////////////

/// Load processes (stored as TBF objects in flash) into runnable process
/// structures stored in the `procs` array and mark all successfully loaded
/// processes as runnable. This method does not check the cryptographic
/// credentials of TBF objects. Platforms for which code size is tight and do
/// not need to check TBF credentials can call this method because it results in
/// a smaller kernel, as it does not invoke the credential checking state
/// machine.
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
    for proc in procs.iter() {
        proc.map(|p| {
            if config::CONFIG.debug_process_credentials {
                debug!("Running {}", p.get_process_name());
            }
        });
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
        let load_binary_result = discover_process_binary(remaining_flash);

        match load_binary_result {
            Ok((new_flash, process_binary)) => {
                remaining_flash = new_flash;

                let load_result = load_process(
                    kernel,
                    chip,
                    process_binary,
                    remaining_memory,
                    ShortId::LocallyUnique,
                    index,
                    fault_policy,
                );
                match load_result {
                    Ok((new_mem, proc)) => {
                        remaining_memory = new_mem;
                        match proc {
                            Some(p) => {
                                if config::CONFIG.debug_load_processes {
                                    debug!("Loaded process {}", p.get_process_name())
                                }
                                procs[index] = proc;
                                index += 1;
                            }
                            None => {
                                if config::CONFIG.debug_load_processes {
                                    debug!("No process loaded.");
                                }
                            }
                        }
                    }
                    Err((new_mem, err)) => {
                        remaining_memory = new_mem;
                        if config::CONFIG.debug_load_processes {
                            debug!("Processes load error: {:?}.", err);
                        }
                    }
                }
            }
            Err((new_flash, err)) => {
                remaining_flash = new_flash;
                match err {
                    ProcessBinaryError::NotEnoughFlash | ProcessBinaryError::TbfHeaderNotFound => {
                        if config::CONFIG.debug_load_processes {
                            debug!("No more processes to load: {:?}.", err);
                        }
                        // No more processes to load.
                        break;
                    }

                    ProcessBinaryError::TbfHeaderParseFailure(_)
                    | ProcessBinaryError::IncompatibleKernelVersion { .. }
                    | ProcessBinaryError::IncorrectFlashAddress { .. }
                    | ProcessBinaryError::NotEnabledProcess
                    | ProcessBinaryError::Padding => {
                        // Skip this binary and move to the next one.
                        continue;
                    }
                }
            }
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// HELPER FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

/// Find a process binary stored at the beginning of `flash` and create a
/// `ProcessBinary` object if the process is viable to run on this kernel.
fn discover_process_binary(
    flash: &'static [u8],
) -> Result<(&'static [u8], ProcessBinary), (&'static [u8], ProcessBinaryError)> {
    if config::CONFIG.debug_load_processes {
        debug!(
            "Loading process binary from flash={:#010X}-{:#010X}",
            flash.as_ptr() as usize,
            flash.as_ptr() as usize + flash.len() - 1
        );
    }

    // If this fails, not enough remaining flash to check for an app.
    let test_header_slice = flash
        .get(0..8)
        .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

    // Pass the first eight bytes to tbfheader to parse out the length of
    // the tbf header and app. We then use those values to see if we have
    // enough flash remaining to parse the remainder of the header.
    //
    // Start by converting [u8] to [u8; 8].
    let header = test_header_slice
        .try_into()
        .or(Err((flash, ProcessBinaryError::NotEnoughFlash)))?;

    let (version, header_length, app_length) =
        match tock_tbf::parse::parse_tbf_header_lengths(header) {
            Ok((v, hl, el)) => (v, hl, el),
            Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(app_length)) => {
                // If we could not parse the header, then we want to skip over
                // this app and look for the next one.
                (0, 0, app_length)
            }
            Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                // Since Tock apps use a linked list, it is very possible the
                // header we started to parse is intentionally invalid to signal
                // the end of apps. This is ok and just means we have finished
                // loading apps.
                return Err((flash, ProcessBinaryError::TbfHeaderNotFound));
            }
        };

    // Now we can get a slice which only encompasses the length of flash
    // described by this tbf header.  We will either parse this as an actual
    // app, or skip over this region.
    let app_flash = flash
        .get(0..app_length as usize)
        .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

    // Advance the flash slice for process discovery beyond this last entry.
    // This will be the start of where we look for a new process since Tock
    // processes are allocated back-to-back in flash.
    let remaining_flash = flash
        .get(app_flash.len()..)
        .ok_or((flash, ProcessBinaryError::NotEnoughFlash))?;

    let pb = ProcessBinary::create(app_flash, header_length as usize, version, true)
        .map_err(|e| (remaining_flash, e))?;

    Ok((remaining_flash, pb))
}

/// Load a process stored as a TBF process binary with `app_memory` as the RAM
/// pool that its RAM should be allocated from. Returns `Ok` if the process
/// object was created, `Err` with a relevant error if the process object could
/// not be created.
fn load_process<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    process_binary: ProcessBinary,
    app_memory: &'static mut [u8],
    app_id: ShortId,
    index: usize,
    fault_policy: &'static dyn ProcessFaultPolicy,
) -> Result<(&'static mut [u8], Option<&'static dyn Process>), (&'static mut [u8], ProcessLoadError)>
{
    if config::CONFIG.debug_load_processes {
        debug!(
            "Loading: process flash={:#010X}-{:#010X} ram={:#010X}-{:#010X}",
            process_binary.flash.as_ptr() as usize,
            process_binary.flash.as_ptr() as usize + process_binary.flash.len() - 1,
            app_memory.as_ptr() as usize,
            app_memory.as_ptr() as usize + app_memory.len() - 1
        );
    }

    // Need to reassign remaining_memory in every iteration so the compiler
    // knows it will not be re-borrowed.
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
            process_binary,
            app_memory,
            fault_policy,
            app_id,
            index,
        )
        .map_err(|(e, memory)| (memory, e))?
    };

    process_option.map(|process| {
        if config::CONFIG.debug_load_processes {
            debug!(
                "Loading: {} [{}] flash={:#010X}-{:#010X} ram={:#010X}-{:#010X}",
                process.get_process_name(),
                index,
                process.get_addresses().flash_start,
                process.get_addresses().flash_end,
                process.get_addresses().sram_start,
                process.get_addresses().sram_end - 1,
            );
        }
    });

    Ok((unused_memory, process_option))
}

////////////////////////////////////////////////////////////////////////////////
// ASYNCHRONOUS PROCESS LOADING
////////////////////////////////////////////////////////////////////////////////

/// Client for asynchronous process loading.
///
/// This supports a client that is notified after trying to load each process in
/// flash. Also there is a callback for after all processes have been
/// discovered.
pub trait ProcessLoadingAsyncClient {
    /// A process was successfully found in flash, checked, and loaded into a
    /// `ProcessStandard` object.
    fn process_loaded(&self, result: Result<(), ProcessLoadError>);

    /// There are no more processes in flash to be loaded.
    fn process_loading_finished(&self);
}

/// Asynchronous process loading.
///
/// Machines which implement this trait perform asynchronous process loading and
/// signal completion through `ProcessLoadingAsyncClient`.
///
/// Various process loaders may exist. This includes a loader from a MCU's
/// integrated flash, or a loader from an external flash chip.
pub trait ProcessLoadingAsync<'a> {
    /// Set the client to receive callbacks about process loading and when
    /// process loading has finished.
    fn set_client(&self, client: &'a dyn ProcessLoadingAsyncClient);

    /// Set the credential checking policy for the loader.
    fn set_policy(&self, policy: &'a dyn AppIdPolicy);

    /// Start the process loading operation.
    fn start(&self);
}

/// Operating mode of the loader.
#[derive(Clone, Copy)]
enum SequentialProcessLoaderMachineState {
    /// Phase of discovering `ProcessBinary` objects in flash.
    DiscoverProcessBinaries,
    /// Phase of loading `ProcessBinary`s into `Process`s.
    LoadProcesses,
}

/// A machine for loading processes stored sequentially in a region of flash.
///
/// Load processes (stored as TBF objects in flash) into runnable process
/// structures stored in the `procs` array. This machine scans the footers in
/// the TBF for cryptographic credentials for binary integrity, passing them to
/// the checker to decide whether the process has sufficient credentials to run.
pub struct SequentialProcessLoaderMachine<'a, C: Chip + 'static> {
    /// Client to notify as processes are loaded and process loading finishes.
    client: OptionalCell<&'a dyn ProcessLoadingAsyncClient>,
    /// Machine to use to check process credentials.
    checker: &'static ProcessCheckerMachine,
    /// Array of stored process references for loaded processes.
    procs: MapCell<&'static mut [Option<&'static dyn Process>]>,
    /// Array to store `ProcessBinary`s after checking credentials.
    proc_binaries: MapCell<&'static mut [Option<ProcessBinary>]>,
    /// Flash memory region to load processes from.
    flash: Cell<&'static [u8]>,
    /// Memory available to assign to applications.
    app_memory: Cell<&'static mut [u8]>,
    /// Mechanism for generating async callbacks.
    deferred_call: DeferredCall,
    /// Reference to the kernel object for creating Processes.
    kernel: &'static Kernel,
    /// Reference to the Chip object for creating Processes.
    chip: &'static C,
    /// The policy to use when determining ShortIds and process uniqueness.
    policy: OptionalCell<&'a dyn AppIdPolicy>,
    /// The fault policy to assign to each created Process.
    fault_policy: &'static dyn ProcessFaultPolicy,
    /// Current mode of the loading machine.
    state: OptionalCell<SequentialProcessLoaderMachineState>,
}

impl<'a, C: Chip> SequentialProcessLoaderMachine<'a, C> {
    /// This function is made `pub` so that board files can use it, but loading
    /// processes from slices of flash an memory is fundamentally unsafe.
    /// Therefore, we require the `ProcessManagementCapability` to call this
    /// function.
    pub fn new(
        checker: &'static ProcessCheckerMachine,
        procs: &'static mut [Option<&'static dyn Process>],
        proc_binaries: &'static mut [Option<ProcessBinary>],
        kernel: &'static Kernel,
        chip: &'static C,
        flash: &'static [u8],
        app_memory: &'static mut [u8],
        fault_policy: &'static dyn ProcessFaultPolicy,
        policy: &'static dyn AppIdPolicy,
        _capability_management: &dyn ProcessManagementCapability,
    ) -> Self {
        Self {
            deferred_call: DeferredCall::new(),
            checker,
            client: OptionalCell::empty(),
            procs: MapCell::new(procs),
            proc_binaries: MapCell::new(proc_binaries),
            kernel,
            chip,
            flash: Cell::new(flash),
            app_memory: Cell::new(app_memory),
            policy: OptionalCell::new(policy),
            fault_policy,
            state: OptionalCell::empty(),
        }
    }

    /// Find a slot in the `PROCESSES` array to store this process.
    fn find_open_process_slot(&self) -> Option<usize> {
        self.procs.map_or(None, |procs| {
            for (i, p) in procs.iter().enumerate() {
                if p.is_none() {
                    return Some(i);
                }
            }
            None
        })
    }

    /// Find a slot in the `PROCESS_BINARIES` array to store this process.
    fn find_open_process_binary_slot(&self) -> Option<usize> {
        self.proc_binaries.map_or(None, |proc_bins| {
            for (i, p) in proc_bins.iter().enumerate() {
                if p.is_none() {
                    return Some(i);
                }
            }
            None
        })
    }

    fn load_and_check(&self) {
        let ret = self.discover_process_binary();
        match ret {
            Ok(pb) => match self.checker.check(pb) {
                Ok(()) => {}
                Err(e) => {
                    self.client.map(|client| {
                        client.process_loaded(Err(ProcessLoadError::CheckError(e)));
                    });
                }
            },
            Err(ProcessBinaryError::NotEnoughFlash)
            | Err(ProcessBinaryError::TbfHeaderNotFound) => {
                // These two errors occur when there are no more app binaries in
                // flash. Now we can move to actually loading process binaries
                // into full processes.

                self.state
                    .set(SequentialProcessLoaderMachineState::LoadProcesses);
                self.deferred_call.set();
            }
            Err(e) => {
                if config::CONFIG.debug_load_processes {
                    debug!("Loading: unable to create ProcessBinary");
                }

                // Other process binary errors indicate the process is not
                // compatible. Signal error and try the next item in flash.
                self.client.map(|client| {
                    client.process_loaded(Err(ProcessLoadError::BinaryError(e)));
                });
                self.deferred_call.set();
            }
        }
    }

    /// Try to parse a process binary from flash.
    ///
    /// Returns the process binary object or an error if a valid process
    /// binary could not be extracted.
    fn discover_process_binary(&self) -> Result<ProcessBinary, ProcessBinaryError> {
        let flash = self.flash.get();

        if config::CONFIG.debug_load_processes {
            debug!(
                "Loading process binary from flash={:#010X}-{:#010X}",
                flash.as_ptr() as usize,
                flash.as_ptr() as usize + flash.len() - 1
            );
        }

        // If this fails, not enough remaining flash to check for an app.
        let test_header_slice = flash.get(0..8).ok_or(ProcessBinaryError::NotEnoughFlash)?;

        // Pass the first eight bytes to tbfheader to parse out the length of
        // the tbf header and app. We then use those values to see if we have
        // enough flash remaining to parse the remainder of the header.
        //
        // Start by converting [u8] to [u8; 8].
        let header = test_header_slice
            .try_into()
            .or(Err(ProcessBinaryError::NotEnoughFlash))?;

        let (version, header_length, app_length) =
            match tock_tbf::parse::parse_tbf_header_lengths(header) {
                Ok((v, hl, el)) => (v, hl, el),
                Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(app_length)) => {
                    // If we could not parse the header, then we want to skip over
                    // this app and look for the next one.
                    (0, 0, app_length)
                }
                Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                    // Since Tock apps use a linked list, it is very possible the
                    // header we started to parse is intentionally invalid to signal
                    // the end of apps. This is ok and just means we have finished
                    // loading apps.
                    return Err(ProcessBinaryError::TbfHeaderNotFound);
                }
            };

        // Now we can get a slice which only encompasses the length of flash
        // described by this tbf header.  We will either parse this as an actual
        // app, or skip over this region.
        let app_flash = flash
            .get(0..app_length as usize)
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;

        // Advance the flash slice for process discovery beyond this last entry.
        // This will be the start of where we look for a new process since Tock
        // processes are allocated back-to-back in flash.
        let remaining_flash = flash
            .get(app_flash.len()..)
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;
        self.flash.set(remaining_flash);

        let pb = ProcessBinary::create(app_flash, header_length as usize, version, true)?;

        Ok(pb)
    }

    /// Create process objects from the discovered process binaries.
    ///
    /// This verifies that the discovered processes are valid to run.
    fn load_process_objects(&self) -> Result<(), ()> {
        let proc_binaries = self.proc_binaries.take().ok_or(())?;
        let proc_binaries_len = proc_binaries.len();

        // Iterate all process binary entries.
        for i in 0..proc_binaries_len {
            // We are either going to load this process binary or discard it, so
            // we can use `take()` here.
            if let Some(process_binary) = proc_binaries[i].take() {
                // We assume the process can be loaded. This is not the case
                // if there is a conflicting process.
                let mut ok_to_load = true;

                // Start by iterating all other process binaries and seeing
                // if any are in conflict (same AppID with newer version).
                for j in 0..proc_binaries_len {
                    match &proc_binaries[j] {
                        Some(other_process_binary) => {
                            let blocked = self
                                .is_blocked_from_loading_by(&process_binary, other_process_binary);

                            if blocked {
                                ok_to_load = false;
                                break;
                            }
                        }
                        None => {}
                    }
                }

                // Go to next ProcessBinary if we cannot load this process.
                if !ok_to_load {
                    continue;
                }

                // Now scan the already loaded processes and make sure this
                // doesn't conflict with any of those. Since those processes
                // are already loaded, we just need to check if this process
                // binary has the same AppID as an already loaded process.
                self.procs.map(|procs| {
                    for proc in procs.iter() {
                        match proc {
                            Some(p) => {
                                let blocked =
                                    self.is_blocked_from_loading_by_process(&process_binary, *p);

                                if blocked {
                                    ok_to_load = false;
                                    break;
                                }
                            }
                            None => {}
                        }
                    }
                });

                if !ok_to_load {
                    continue;
                }

                // If we get here it is ok to load the process.
                match self.find_open_process_slot() {
                    Some(index) => {
                        // Calculate the ShortId for this new process.
                        let short_app_id = self.policy.map_or(ShortId::LocallyUnique, |policy| {
                            policy.to_short_id(&process_binary)
                        });

                        // Try to create a `Process` object.
                        let load_result = load_process(
                            self.kernel,
                            self.chip,
                            process_binary,
                            self.app_memory.take(),
                            short_app_id,
                            index,
                            self.fault_policy,
                        );
                        match load_result {
                            Ok((new_mem, proc)) => {
                                self.app_memory.set(new_mem);
                                match proc {
                                    Some(p) => {
                                        if config::CONFIG.debug_load_processes {
                                            debug!(
                                                "Loading: Loaded process {}",
                                                p.get_process_name()
                                            )
                                        }

                                        // Store the `ProcessStandard` object in the `PROCESSES`
                                        // array.
                                        self.procs.map(|procs| {
                                            procs[index] = proc;
                                        });
                                        // Notify the client the process was loaded
                                        // successfully.
                                        self.client.map(|client| {
                                            client.process_loaded(Ok(()));
                                        });
                                    }
                                    None => {
                                        if config::CONFIG.debug_load_processes {
                                            debug!("No process loaded.");
                                        }
                                    }
                                }
                            }
                            Err((new_mem, err)) => {
                                self.app_memory.set(new_mem);
                                if config::CONFIG.debug_load_processes {
                                    debug!("Could not load process: {:?}.", err);
                                }

                                self.client.map(|client| {
                                    client.process_loaded(Err(err));
                                });
                            }
                        }
                    }
                    None => {
                        // Nowhere to store the process.
                        self.client.map(|client| {
                            client.process_loaded(Err(ProcessLoadError::NoProcessSlot));
                        });
                    }
                }
            }
        }
        self.proc_binaries.put(proc_binaries);

        // We have iterated all discovered `ProcessBinary`s and loaded what we
        // could so now we can signal that process loading is finished.
        self.client.map(|client| {
            client.process_loading_finished();
        });

        self.state.clear();
        Ok(())
    }

    /// Check if `pb1` is blocked from running by `pb2`.
    ///
    /// `pb2` blocks `pb1` if:
    ///
    /// - They both have the same AppID or they both have the same ShortId, and
    /// - `pb2` has a higher version number.
    fn is_blocked_from_loading_by(&self, pb1: &ProcessBinary, pb2: &ProcessBinary) -> bool {
        let same_app_id = self
            .policy
            .map_or(false, |policy| !policy.different_identifier(pb1, pb2));
        let same_short_app_id = self.policy.map_or(false, |policy| {
            policy.to_short_id(pb1) == policy.to_short_id(pb2)
        });
        let other_newer = pb2.header.get_binary_version() > pb1.header.get_binary_version();

        let blocks = (same_app_id || same_short_app_id) && other_newer;

        if config::CONFIG.debug_process_credentials {
            debug!(
                "Loading: ProcessBinary {}({:#02x}) does{} block {}({:#02x})",
                pb2.header.get_package_name().unwrap_or(""),
                pb2.flash.as_ptr() as usize,
                if blocks { " not" } else { "" },
                pb1.header.get_package_name().unwrap_or(""),
                pb1.flash.as_ptr() as usize,
            );
        }

        blocks
    }

    /// Check if `pb` is blocked from running by `process`.
    ///
    /// `process` blocks `pb` if:
    ///
    /// - They both have the same AppID, or
    /// - They both have the same ShortId
    ///
    /// Since `process` is already loaded, we only have to enforce the AppID and
    /// ShortId uniqueness guarantees.
    fn is_blocked_from_loading_by_process(
        &self,
        pb: &ProcessBinary,
        process: &dyn Process,
    ) -> bool {
        let same_app_id = self.policy.map_or(false, |policy| {
            !policy.different_identifier_process(pb, process)
        });
        let same_short_app_id = self.policy.map_or(false, |policy| {
            policy.to_short_id(pb) == process.short_app_id()
        });

        let blocks = same_app_id || same_short_app_id;

        if config::CONFIG.debug_process_credentials {
            debug!(
                "Loading: Process {}({:#02x}) does{} block {}({:#02x})",
                process.get_process_name(),
                process.get_addresses().flash_start,
                if blocks { " not" } else { "" },
                pb.header.get_package_name().unwrap_or(""),
                pb.flash.as_ptr() as usize,
            );
        }

        blocks
    }
}

impl<'a, C: Chip> ProcessLoadingAsync<'a> for SequentialProcessLoaderMachine<'a, C> {
    fn set_client(&self, client: &'a dyn ProcessLoadingAsyncClient) {
        self.client.set(client);
    }

    fn set_policy(&self, policy: &'a dyn AppIdPolicy) {
        self.policy.replace(policy);
    }

    fn start(&self) {
        self.state
            .set(SequentialProcessLoaderMachineState::DiscoverProcessBinaries);
        // Start an asynchronous flow so we can issue a callback on error.
        self.deferred_call.set();
    }
}

impl<'a, C: Chip> DeferredCallClient for SequentialProcessLoaderMachine<'a, C> {
    fn handle_deferred_call(&self) {
        // We use deferred calls to start the operation in the async loop.
        match self.state.get() {
            Some(SequentialProcessLoaderMachineState::DiscoverProcessBinaries) => {
                self.load_and_check();
            }
            Some(SequentialProcessLoaderMachineState::LoadProcesses) => {
                let ret = self.load_process_objects();
                match ret {
                    Ok(()) => {}
                    Err(()) => {
                        // If this failed for some reason, we still need to
                        // signal that process loading has finished.
                        self.client.map(|client| {
                            client.process_loading_finished();
                        });
                    }
                }
            }
            None => {}
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a, C: Chip> crate::process_checker::ProcessCheckerMachineClient
    for SequentialProcessLoaderMachine<'a, C>
{
    fn done(
        &self,
        process_binary: ProcessBinary,
        result: Result<(), crate::process_checker::ProcessCheckError>,
    ) {
        // Check if this process was approved by the checker.
        match result {
            Ok(()) => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "Loading: Check succeeded for process {}",
                        process_binary.header.get_package_name().unwrap_or("")
                    );
                }
                // Save the checked process binary now that we know it is valid.
                match self.find_open_process_binary_slot() {
                    Some(index) => {
                        self.proc_binaries.map(|proc_binaries| {
                            proc_binaries[index] = Some(process_binary);
                        });
                    }
                    None => {
                        self.client.map(|client| {
                            client.process_loaded(Err(ProcessLoadError::NoProcessSlot));
                        });
                    }
                }
            }
            Err(e) => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "Loading: Process {} check failed {:?}",
                        process_binary.header.get_package_name().unwrap_or(""),
                        e
                    );
                }
                // Signal error and call try next
                self.client.map(|client| {
                    client.process_loaded(Err(ProcessLoadError::CheckError(e)));
                });
            }
        }

        // Try to load the next process in flash.
        self.deferred_call.set();
    }
}
