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
use crate::process_checker::AcceptedCredential;
use crate::process_checker::{AppIdPolicy, ProcessCheckError, ProcessCheckerMachine};
use crate::process_policies::ProcessFaultPolicy;
use crate::process_policies::ProcessStandardStoragePermissionsPolicy;
use crate::process_standard::ProcessStandard;
use crate::process_standard::{ProcessStandardDebug, ProcessStandardDebugFull};
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

/// Load processes into runnable process structures.
///
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
    load_processes_from_flash::<C, ProcessStandardDebugFull>(
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
fn load_processes_from_flash<C: Chip, D: ProcessStandardDebug + 'static>(
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

                let load_result = load_process::<C, D>(
                    kernel,
                    chip,
                    process_binary,
                    remaining_memory,
                    ShortId::LocallyUnique,
                    index,
                    fault_policy,
                    &(),
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
                        if config::CONFIG.debug_load_processes {
                            debug!("Unable to use process binary: {:?}.", err);
                        }

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
            "Looking for process binary in flash={:#010X}-{:#010X}",
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
fn load_process<C: Chip, D: ProcessStandardDebug>(
    kernel: &'static Kernel,
    chip: &'static C,
    process_binary: ProcessBinary,
    app_memory: &'static mut [u8],
    app_id: ShortId,
    index: usize,
    fault_policy: &'static dyn ProcessFaultPolicy,
    storage_policy: &'static dyn ProcessStandardStoragePermissionsPolicy<C, D>,
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
        ProcessStandard::<C, D>::create(
            kernel,
            chip,
            process_binary,
            app_memory,
            fault_policy,
            storage_policy,
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
    /// Phase of loading `ProcessBinary`s into `Process`es.
    LoadProcesses,
}

/// Operating mode of the sequential process loader.
///
/// The loader supports loading processes from flash at boot, and loading processes
/// that were written to flash dynamically at runtime. Most of the internal logic is the
/// same (and therefore reused), but we need to track which mode of operation the
/// loader is in.
#[derive(Clone, Copy)]
enum SequentialProcessLoaderMachineRunMode {
    /// The loader was called by a board's main function at boot.
    BootMode,
    /// The loader was called by a dynamic process loader at runtime.
    RuntimeMode,
}

/// Enum to hold the padding requirements for a new application.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum PaddingRequirement {
    #[default]
    None,
    PrePad,
    PostPad,
    PreAndPostPad,
}

/// A machine for loading processes stored sequentially in a region of flash.
///
/// Load processes (stored as TBF objects in flash) into runnable process
/// structures stored in the `procs` array. This machine scans the footers in
/// the TBF for cryptographic credentials for binary integrity, passing them to
/// the checker to decide whether the process has sufficient credentials to run.
pub struct SequentialProcessLoaderMachine<'a, C: Chip + 'static, D: ProcessStandardDebug + 'static>
{
    /// Client to notify as processes are loaded and process loading finishes after boot.
    boot_client: OptionalCell<&'a dyn ProcessLoadingAsyncClient>,
    /// Client to notify as processes are loaded and process loading finishes during runtime.
    runtime_client: OptionalCell<&'a dyn ProcessLoadingAsyncClient>,
    /// Machine to use to check process credentials.
    checker: &'static ProcessCheckerMachine,
    /// Array of stored process references for loaded processes.
    procs: MapCell<&'static mut [Option<&'static dyn Process>]>,
    /// Array to store `ProcessBinary`s after checking credentials.
    proc_binaries: MapCell<&'static mut [Option<ProcessBinary>]>,
    /// Total available flash for process binaries on this board.
    flash_bank: Cell<&'static [u8]>,
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
    /// The storage permissions policy to assign to each created Process.
    storage_policy: &'static dyn ProcessStandardStoragePermissionsPolicy<C, D>,
    /// Current mode of the loading machine.
    state: OptionalCell<SequentialProcessLoaderMachineState>,
    /// Current operating mode of the loading machine.
    run_mode: OptionalCell<SequentialProcessLoaderMachineRunMode>,
}

impl<'a, C: Chip, D: ProcessStandardDebug> SequentialProcessLoaderMachine<'a, C, D> {
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
        storage_policy: &'static dyn ProcessStandardStoragePermissionsPolicy<C, D>,
        policy: &'static dyn AppIdPolicy,
        _capability_management: &dyn ProcessManagementCapability,
    ) -> Self {
        Self {
            deferred_call: DeferredCall::new(),
            checker,
            boot_client: OptionalCell::empty(),
            runtime_client: OptionalCell::empty(),
            run_mode: OptionalCell::empty(),
            procs: MapCell::new(procs),
            proc_binaries: MapCell::new(proc_binaries),
            kernel,
            chip,
            flash_bank: Cell::new(flash),
            flash: Cell::new(flash),
            app_memory: Cell::new(app_memory),
            policy: OptionalCell::new(policy),
            fault_policy,
            storage_policy,
            state: OptionalCell::empty(),
        }
    }

    /// Set the runtime client to receive callbacks about process loading and when
    /// process loading has finished.
    pub fn set_runtime_client(&self, client: &'a dyn ProcessLoadingAsyncClient) {
        self.runtime_client.set(client);
    }

    /// Find the current active client based on the operation mode.
    fn get_current_client(&self) -> Option<&dyn ProcessLoadingAsyncClient> {
        match self.run_mode.get()? {
            SequentialProcessLoaderMachineRunMode::BootMode => self.boot_client.get(),
            SequentialProcessLoaderMachineRunMode::RuntimeMode => self.runtime_client.get(),
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
                    self.get_current_client().map(|client| {
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
                    debug!("Loading: unable to create ProcessBinary: {:?}", e);
                }

                // Other process binary errors indicate the process is not
                // compatible. Signal error and try the next item in flash.
                self.get_current_client().map(|client| {
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

        match discover_process_binary(flash) {
            Ok((remaining_flash, pb)) => {
                self.flash.set(remaining_flash);
                Ok(pb)
            }

            Err((remaining_flash, err)) => {
                self.flash.set(remaining_flash);
                Err(err)
            }
        }
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
                for proc_bin in proc_binaries.iter() {
                    match proc_bin {
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
                            self.storage_policy,
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
                                        self.get_current_client().map(|client| {
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
                                self.get_current_client().map(|client| {
                                    client.process_loaded(Err(err));
                                });
                            }
                        }
                    }
                    None => {
                        // Nowhere to store the process.
                        self.get_current_client().map(|client| {
                            client.process_loaded(Err(ProcessLoadError::NoProcessSlot));
                        });
                    }
                }
            }
        }
        self.proc_binaries.put(proc_binaries);

        // We have iterated all discovered `ProcessBinary`s and loaded what we
        // could so now we can signal that process loading is finished.
        self.get_current_client().map(|client| {
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

    ////////////////////////////////////////////////////////////////////////////////
    // DYNAMIC PROCESS LOADING HELPERS
    ////////////////////////////////////////////////////////////////////////////////

    /// Scan the entire flash to populate lists of existing binaries addresses.
    fn scan_flash_for_process_binaries(
        &self,
        flash: &'static [u8],
        process_binaries_start_addresses: &mut [usize],
        process_binaries_end_addresses: &mut [usize],
    ) -> Result<(), ()> {
        fn inner_function(
            flash: &'static [u8],
            process_binaries_start_addresses: &mut [usize],
            process_binaries_end_addresses: &mut [usize],
        ) -> Result<(), ProcessBinaryError> {
            let flash_end = flash.as_ptr() as usize + flash.len() - 1;
            let mut addresses = flash.as_ptr() as usize;
            let mut index: usize = 0;

            while addresses < flash_end {
                let flash_offset = addresses - flash.as_ptr() as usize;

                let test_header_slice = flash
                    .get(flash_offset..flash_offset + 8)
                    .ok_or(ProcessBinaryError::NotEnoughFlash)?;

                let header = test_header_slice
                    .try_into()
                    .or(Err(ProcessBinaryError::NotEnoughFlash))?;

                let (_version, header_length, app_length) =
                    match tock_tbf::parse::parse_tbf_header_lengths(header) {
                        Ok((v, hl, el)) => (v, hl, el),
                        Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(app_length)) => {
                            (0, 0, app_length)
                        }
                        Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                            return Ok(());
                        }
                    };

                let app_flash = flash
                    .get(flash_offset..flash_offset + app_length as usize)
                    .ok_or(ProcessBinaryError::NotEnoughFlash)?;

                let app_header = flash
                    .get(flash_offset..flash_offset + header_length as usize)
                    .ok_or(ProcessBinaryError::NotEnoughFlash)?;

                let remaining_flash = flash
                    .get(flash_offset + app_flash.len()..)
                    .ok_or(ProcessBinaryError::NotEnoughFlash)?;

                // Get the rest of the header. The `remaining_header` variable
                // will continue to hold the remainder of the header we have
                // not processed.
                let remaining_header = app_header
                    .get(16..)
                    .ok_or(ProcessBinaryError::NotEnoughFlash)?;

                if remaining_header.len() == 0 {
                    // This is a padding app.
                    if config::CONFIG.debug_load_processes {
                        debug!("Is padding!");
                    }
                } else {
                    // This is an app binary, add it to the pb arrays.
                    process_binaries_start_addresses[index] = app_flash.as_ptr() as usize;
                    process_binaries_end_addresses[index] =
                        app_flash.as_ptr() as usize + app_length as usize;

                    if config::CONFIG.debug_load_processes {
                        debug!(
                            "[Metadata] Process binary start address at index {}: {:#010x}, with end_address {:#010x}",
                            index,
                            process_binaries_start_addresses[index],
                            process_binaries_end_addresses[index]
                        );
                    }
                    index += 1;
                    if index > process_binaries_start_addresses.len() - 1 {
                        return Err(ProcessBinaryError::NotEnoughFlash);
                    }
                }
                addresses = remaining_flash.as_ptr() as usize;
            }

            Ok(())
        }

        inner_function(
            flash,
            process_binaries_start_addresses,
            process_binaries_end_addresses,
        )
        .or(Err(()))
    }

    /// Helper function to find the next potential aligned address for the
    /// new app with size `app_length` assuming Cortex-M alignment rules.
    fn find_next_cortex_m_aligned_address(&self, address: usize, app_length: usize) -> usize {
        let remaining = address % app_length;
        if remaining == 0 {
            address
        } else {
            address + (app_length - remaining)
        }
    }

    /// Function to compute the address for a new app with size `app_size`.
    fn compute_new_process_binary_address(
        &self,
        app_size: usize,
        process_binaries_start_addresses: &mut [usize],
        process_binaries_end_addresses: &mut [usize],
    ) -> usize {
        let mut start_count = 0;
        let mut end_count = 0;

        // Remove zeros from addresses in place.
        for i in 0..process_binaries_start_addresses.len() {
            if process_binaries_start_addresses[i] != 0 {
                process_binaries_start_addresses[start_count] = process_binaries_start_addresses[i];
                start_count += 1;
            }
        }

        for i in 0..process_binaries_end_addresses.len() {
            if process_binaries_end_addresses[i] != 0 {
                process_binaries_end_addresses[end_count] = process_binaries_end_addresses[i];
                end_count += 1;
            }
        }

        // If there is only one application in flash:
        if start_count == 1 {
            let potential_address = self
                .find_next_cortex_m_aligned_address(process_binaries_end_addresses[0], app_size);
            return potential_address;
        }

        // Otherwise, iterate through the sorted start and end addresses to find gaps for the new app.
        for i in 0..start_count - 1 {
            let gap_start = process_binaries_end_addresses[i];
            let gap_end = process_binaries_start_addresses[i + 1];

            // Ensure gap_end is valid (skip zeros - these indicate there are no process binaries).
            if gap_end == 0 {
                continue;
            }

            // If there is a valid gap, i.e., (gap_end > gap_start), check alignment.
            if gap_end > gap_start {
                let potential_address =
                    self.find_next_cortex_m_aligned_address(gap_start, app_size);
                if potential_address + app_size < gap_end {
                    return potential_address;
                }
            }
        }
        // If no gaps found, check after the last app.
        let last_app_end_address = process_binaries_end_addresses[end_count - 1];
        let potential_address =
            self.find_next_cortex_m_aligned_address(last_app_end_address, app_size);
        potential_address
    }

    /// This function checks if there is a need to pad either before or after
    /// the new app to preserve the linked list.
    ///
    /// When do we pad?
    ///
    /// 1. When there is a binary  located in flash after the new app but
    ///    not immediately after, we need to add padding between the new
    ///    app and the existing app.
    /// 2. Due to MPU alignment, the new app may be similarly placed not
    ///    immediately after an existing process, in that case, we need to add
    ///    padding between the previous app and the new app.
    /// 3. If both the above conditions are met, we add both a prepadding and a
    ///    postpadding.
    /// 4. If either of these conditions are not met, we don't pad.
    ///
    /// Change checks against process binaries instead of processes?
    fn compute_padding_requirement_and_neighbors(
        &self,
        new_app_start_address: usize,
        app_length: usize,
        process_binaries_start_addresses: &[usize],
        process_binaries_end_addresses: &[usize],
    ) -> (PaddingRequirement, usize, usize) {
        // The end address of our newly loaded application.
        let new_app_end_address = new_app_start_address + app_length;
        // To store the address until which we need to write the padding app.
        let mut next_app_start_addr = 0;
        // To store the address from which we need to write the padding app.
        let mut previous_app_end_addr = 0;
        let mut padding_requirement: PaddingRequirement = PaddingRequirement::None;

        // We compute the closest neighbor to our app such that:
        //
        // 1. If the new app is placed in between two existing binaries, we
        //    compute the closest located binaries.
        // 2. Once we compute these values, we determine if we need to write a
        //    pre pad header, or a post pad header, or both.
        // 3. If there are no apps after ours in the process binary array, we don't
        //    do anything.

        // Postpad requirement.
        if let Some(next_closest_neighbor) = process_binaries_start_addresses
            .iter()
            .filter(|&&x| x > new_app_end_address - 1)
            .min()
        {
            // We found the next closest app in flash.
            next_app_start_addr = *next_closest_neighbor;
            if next_app_start_addr != 0 {
                padding_requirement = PaddingRequirement::PostPad;
            }
        } else {
            if config::CONFIG.debug_load_processes {
                debug!("No App Found after the new app so not adding post padding.");
            }
        }

        // Prepad requirement.
        if let Some(previous_closest_neighbor) = process_binaries_end_addresses
            .iter()
            .filter(|&&x| x < new_app_start_address + 1)
            .max()
        {
            // We found the previous closest app in flash.
            previous_app_end_addr = *previous_closest_neighbor;
            if new_app_start_address - previous_app_end_addr != 0 {
                if padding_requirement == PaddingRequirement::PostPad {
                    padding_requirement = PaddingRequirement::PreAndPostPad;
                } else {
                    padding_requirement = PaddingRequirement::PrePad;
                }
            }
        } else {
            if config::CONFIG.debug_load_processes {
                debug!("No Previous App Found, so not padding before the new app.");
            }
        }
        (
            padding_requirement,
            previous_app_end_addr,
            next_app_start_addr,
        )
    }

    /// This function scans flash, checks for, and returns an address that follows alignment rules given
    /// an app size of `new_app_size`.
    fn check_flash_for_valid_address(
        &self,
        new_app_size: usize,
        pb_start_address: &mut [usize],
        pb_end_address: &mut [usize],
    ) -> Result<usize, ProcessBinaryError> {
        let total_flash = self.flash_bank.get();
        let total_flash_start = total_flash.as_ptr() as usize;
        let total_flash_end = total_flash_start + total_flash.len() - 1;

        match self.scan_flash_for_process_binaries(total_flash, pb_start_address, pb_end_address) {
            Ok(()) => {
                if config::CONFIG.debug_load_processes {
                    debug!("Successfully scanned flash");
                }
                let new_app_address = self.compute_new_process_binary_address(
                    new_app_size,
                    pb_start_address,
                    pb_end_address,
                );
                if new_app_address + new_app_size - 1 > total_flash_end {
                    Err(ProcessBinaryError::NotEnoughFlash)
                } else {
                    Ok(new_app_address)
                }
            }
            Err(()) => Err(ProcessBinaryError::NotEnoughFlash),
        }
    }

    /// Function to check if the object with address `offset` of size `length` lies
    /// within flash bounds.
    pub fn check_if_within_flash_bounds(&self, offset: usize, length: usize) -> bool {
        let flash = self.flash_bank.get();
        let flash_end = flash.as_ptr() as usize + flash.len() - 1;

        (flash_end - offset) >= length
    }

    /// Function to compute an available address for the new application binary.
    pub fn check_flash_for_new_address(
        &self,
        new_app_size: usize,
    ) -> Result<(usize, PaddingRequirement, usize, usize), ProcessBinaryError> {
        const MAX_PROCS: usize = 10;
        let mut pb_start_address: [usize; MAX_PROCS] = [0; MAX_PROCS];
        let mut pb_end_address: [usize; MAX_PROCS] = [0; MAX_PROCS];
        match self.check_flash_for_valid_address(
            new_app_size,
            &mut pb_start_address,
            &mut pb_end_address,
        ) {
            Ok(app_address) => {
                let (pr, prev_app_addr, next_app_addr) = self
                    .compute_padding_requirement_and_neighbors(
                        app_address,
                        new_app_size,
                        &pb_start_address,
                        &pb_end_address,
                    );
                let (padding_requirement, previous_app_end_addr, next_app_start_addr) =
                    (pr, prev_app_addr, next_app_addr);
                Ok((
                    app_address,
                    padding_requirement,
                    previous_app_end_addr,
                    next_app_start_addr,
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Function to check if the app binary at address `app_address` is valid.
    fn check_new_binary_validity(&self, app_address: usize) -> bool {
        let flash = self.flash_bank.get();
        // Pass the first eight bytes of the tbfheader to parse out the
        // length of the tbf header and app. We then use those values to see
        // if we have enough flash remaining to parse the remainder of the
        // header.
        let binary_header = match flash.get(app_address..app_address + 8) {
            Some(slice) if slice.len() == 8 => slice,
            _ => return false, // Ensure exactly 8 bytes are available
        };

        let binary_header_array: &[u8; 8] = match binary_header.try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };

        match tock_tbf::parse::parse_tbf_header_lengths(binary_header_array) {
            Ok((_version, _header_length, _entry_length)) => true,
            Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(_entry_length)) => false,
            Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => false,
        }
    }

    /// Function to start loading the new application at address `app_address` with size
    /// `app_size`.
    pub fn load_new_process_binary(
        &self,
        app_address: usize,
        app_size: usize,
    ) -> Result<(), ProcessLoadError> {
        let flash = self.flash_bank.get();
        let process_address = app_address - flash.as_ptr() as usize;
        let process_flash = flash.get(process_address..process_address + app_size);
        let result = self.check_new_binary_validity(process_address);
        match result {
            true => {
                if let Some(flash) = process_flash {
                    self.flash.set(flash);
                } else {
                    return Err(ProcessLoadError::BinaryError(
                        ProcessBinaryError::TbfHeaderNotFound,
                    ));
                }

                self.state
                    .set(SequentialProcessLoaderMachineState::DiscoverProcessBinaries);

                self.run_mode
                    .set(SequentialProcessLoaderMachineRunMode::RuntimeMode);
                // Start an asynchronous flow so we can issue a callback on error.
                self.deferred_call.set();

                Ok(())
            }
            false => Err(ProcessLoadError::BinaryError(
                ProcessBinaryError::TbfHeaderNotFound,
            )),
        }
    }
}

impl<'a, C: Chip, D: ProcessStandardDebug> ProcessLoadingAsync<'a>
    for SequentialProcessLoaderMachine<'a, C, D>
{
    fn set_client(&self, client: &'a dyn ProcessLoadingAsyncClient) {
        self.boot_client.set(client);
    }

    fn set_policy(&self, policy: &'a dyn AppIdPolicy) {
        self.policy.replace(policy);
    }

    fn start(&self) {
        self.state
            .set(SequentialProcessLoaderMachineState::DiscoverProcessBinaries);
        self.run_mode
            .set(SequentialProcessLoaderMachineRunMode::BootMode);
        // Start an asynchronous flow so we can issue a callback on error.
        self.deferred_call.set();
    }
}

impl<C: Chip, D: ProcessStandardDebug> DeferredCallClient
    for SequentialProcessLoaderMachine<'_, C, D>
{
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
                        self.get_current_client().map(|client| {
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

impl<C: Chip, D: ProcessStandardDebug> crate::process_checker::ProcessCheckerMachineClient
    for SequentialProcessLoaderMachine<'_, C, D>
{
    fn done(
        &self,
        process_binary: ProcessBinary,
        result: Result<Option<AcceptedCredential>, crate::process_checker::ProcessCheckError>,
    ) {
        // Check if this process was approved by the checker.
        match result {
            Ok(optional_credential) => {
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
                            process_binary.credential.insert(optional_credential);
                            proc_binaries[index] = Some(process_binary);
                        });
                    }
                    None => {
                        self.get_current_client().map(|client| {
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
                self.get_current_client().map(|client| {
                    client.process_loaded(Err(ProcessLoadError::CheckError(e)));
                });
            }
        }

        // Try to load the next process in flash.
        self.deferred_call.set();
    }
}
