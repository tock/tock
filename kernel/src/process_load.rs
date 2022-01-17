//! Reading process binaries and loading them into in-memory Tock processes.
//!
//! The process loader is responsible for parsing the binary formats of Tock processes,
//! checking whether they are allowed to be loaded, and if so initializing a process
//! structure to run it.

use core::cell::Cell;
use core::convert::TryInto;
use core::fmt;
use core::slice;

use crate::capabilities::{ProcessApprovalCapability, ProcessManagementCapability};
use crate::config;
use crate::create_capability;
use crate::debug;
use crate::ErrorCode;
use crate::kernel::Kernel;
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::process_checking;
use crate::process_checking::{AppCredentialsChecker};
use crate::process_policies::ProcessFaultPolicy;
use crate::process_standard::ProcessStandard;
use crate::static_init;
use crate::utilities::cells::OptionalCell;



use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfParseError;

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

    /// The process did not contain a credentials which the process binary verifier
    /// accepted and the verifier requires credentials.
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

            ProcessLoadError::IncompatibleKernelVersion { version } => match version {
                Some((major, minor)) => write!(
                    f,
                    "Process is incompatible with the kernel. Running: {}.{}, Requested: {}.{}",
                    crate::MAJOR,
                    crate::MINOR,
                    major,
                    minor
                ),
                None => write!(f, "Process did not provide a TBF kernel version header"),
            },

            ProcessLoadError::InternalError => write!(f, "Error in kernel. Likely a bug."),

            ProcessLoadError::CredentialsNoAccept => write!(f, "No credentials accepted."),
            
            ProcessLoadError::CredentialsReject(index) => write!(f, "Credentials index {} rejected.", index),
        }
    }
}


/// Load processes (stored as TBF objects in flash) into runnable
/// process structures stored in the `procs` array. If a `checker` is
/// passed, this method scans the footers in the TBF for cryptographic
/// credentials for binary integrity, passing them to `checker` to decide
/// whether the process has sufficient credentials to run.
#[inline(always)]
pub fn load_and_check_processes<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8], 
    mut procs: &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
    checker: Option<&'static dyn AppCredentialsChecker>,
    capability_management: &dyn ProcessManagementCapability
) -> Result<(), ProcessLoadError> {
    load_processes(kernel,
                   chip,
                   app_flash,
                   app_memory,
                   &mut procs,
                   fault_policy,
                   capability_management)?;
    let _res = check_processes(procs, checker);
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
/// This function is made `pub` so that board files can use it, but loading
/// processes from slices of flash an memory is fundamentally unsafe. Therefore,
/// we require the `ProcessManagementCapability` to call this function.
///
/// Returns `Ok(())` if process discovery went as expected. Returns a
/// `ProcessLoadError` if something goes wrong during TBF parsing or process
/// creation.
#[inline(always)]
fn load_processes<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &'static mut [u8], 
    procs: &mut &'static mut [Option<&'static dyn Process>],
    fault_policy: &'static dyn ProcessFaultPolicy,
    capability: &dyn ProcessManagementCapability,
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
        let load_result = load_process(kernel,
                                       chip,
                                       remaining_flash,
                                       remaining_memory,
                                       index,
                                       fault_policy,
                                       capability);
        match load_result {
            Ok((new_flash, new_mem, proc)) => {
                remaining_flash = new_flash;
                remaining_memory = new_mem;
                if proc.is_some() {
                    if config::CONFIG.debug_load_processes {
                        proc.map(|p| debug!("Loaded process {}", p.get_process_name()));
                    }
                    procs[index] = proc;
                    index = index + 1;
                }
            },
            Err((_new_flash, _new_mem, _err)) => {
                // No more processes to load.
                break;
            }
        }
    }
    Ok(())
}


/// Use `checker` to transition `procs` from the `Unverified` into the
/// `Unstarted` state (if they pass the checker policy) or
/// `CredentialsFailed` state (if they do not pass the checker
/// policy). If `verifier` is `None` then all processes are
/// automatically verified. Processes that transition into the
/// `Unstarted` state are started by enqueueing a stack frame to run
/// the initialization function as indicated in the TBF header.
#[inline(always)]
fn check_processes(procs: &'static [Option<&'static dyn Process>],
                   checker: Option<&'static dyn AppCredentialsChecker>
) -> Result<(), ProcessLoadError> {
    let capability = create_capability!(ProcessApprovalCapability);

    if checker.is_none() {
        if config::CONFIG.debug_process_credentials {
            debug!("Checking: no checker provided, load and run all processes");
        }
        for proc in procs.iter() {
            let res = proc.map(|p| {
                p.mark_credentials_pass(None, &capability).or(Err(ProcessLoadError::InternalError))?;
                p.enqueue_init_task().or(Err(ProcessLoadError::InternalError))?;
                Ok(())
            });
            if let Some(Err(e)) = res {
                return Err(e);
            }
        }
        Ok(())
    } else {
        checker.map_or(Err(ProcessLoadError::InternalError), |c| {
            #[allow(unused_mut)] // machine doesn't need mut
            let machine = unsafe {static_init!(ProcessCheckerMachine,
                                               ProcessCheckerMachine {
                                                   process: Cell::new(0),
                                                   footer:  Cell::new(0),
                                                   checker: OptionalCell::empty(),
                                                   processes: procs})};
            c.set_client(machine);
            machine.checker.replace(c);
            machine.next()?;
            Ok(())
        })
    }
}

/// Iterates across the `processes` array, checking footers and deciding
/// whether to make them runnable based on the checking policy in `checker`.
/// Starts processes that pass the policy and puts processes that don't
/// pass the policy into the `CredentialsFailed` state.
struct ProcessCheckerMachine {
    process: Cell<usize>,
    footer: Cell<usize>,
    checker: OptionalCell<&'static dyn AppCredentialsChecker<'static>>,
    processes: &'static [Option<&'static dyn Process>],
}

#[derive(Debug)]
enum FooterCheckResult {
    Checking,           // A check has started
    PastLastFooter,     // There are no more footers, no check started
    FooterNotCheckable, // The footer isn't a credential, no check started
    BadFooter,          // The footer is invalid, no check started
    NoProcess,          // No process was provided, no check started
    Error               // An internal error occured, no check started
}
    

impl ProcessCheckerMachine {

    /// Check the next footer of the next process. Returns:
    ///   - Ok(true) if a valid footer was found and is being checked
    ///   - Ok(false) there are no more footers to check
    ///   - Err(r): an error occured and process verification has to stop.
    fn next(&self) -> Result<bool, ProcessLoadError> {
        let capability = create_capability!(ProcessApprovalCapability);
        loop {
            let mut proc_index = self.process.get();
            
            // Find the next process to check. When code completes
            // checking a process, it just increments to the next
            // index. In case the array has None entries or the
            // process array changes under us, don't actually trust
            // this value.
            while proc_index < self.processes.len() &&
                  self.processes[proc_index].is_none() {
                proc_index = proc_index + 1;
                self.process.set(proc_index);
                self.footer.set(0);
            }
            if proc_index >= self.processes.len() {
                // No more processes to check.
                return Ok(false);
            }
            
            let footer_index = self.footer.get();
            // Try to check the next footer.
            let check_result = self
                .checker
                .map_or(FooterCheckResult::Error,
                        |v| check_footer(self.processes[proc_index],
                                         *v,
                                         footer_index));
            if config::CONFIG.debug_process_credentials {
                debug!("Checking: Check status for process {}, footer {}: {:?}",
                       proc_index,
                       footer_index,
                       check_result);
            }
            match check_result {
                FooterCheckResult::Checking => {
                    return Ok(true);
                }
                FooterCheckResult::PastLastFooter => {
                    // We reached the end of the footers without any
                    // credentials or all credentials were Pass: apply
                    // the checker policy to see if the process
                    // should be allowed to run.
                    let requires = self.checker.map_or(false, |v| v.require_credentials());
                    let _res = self.processes[proc_index]
                        .map_or(Err(ProcessLoadError::InternalError), |p| {
                            if requires {
                                if config::CONFIG.debug_process_credentials {
                                    debug!("Checking: required, but all passes, do not run {}", p.get_process_name());
                                }
                                p.mark_credentials_fail(&capability);
                            } else {
                                if config::CONFIG.debug_process_credentials {
                                    debug!("Checking: not required, all passes, run {}", p.get_process_name());
                                }
                                p.mark_credentials_pass(None, &capability).or(Err(ProcessLoadError::InternalError))?;
                                p.enqueue_init_task().or(Err(ProcessLoadError::InternalError))?;
                            }
                            Ok(true)
                        });
                    self.process.set(self.process.get() + 1);
                    self.footer.set(0);
                },
                FooterCheckResult::NoProcess |
                FooterCheckResult::BadFooter => {
                    // Go to next process
                    self.process.set(self.process.get() + 1);
                    self.footer.set(0)                    
                }
                FooterCheckResult::FooterNotCheckable => {
                    // Go to next footer
                    self.footer.set(self.footer.get() + 1);                    
                }
                FooterCheckResult::Error => {
                    return Err(ProcessLoadError::InternalError);
                }
            }
        }
    }
}

// Returns whether a footer is being checked or not, and if not, why.
// Iterates through the footer list until if finds `next_footer` or
// it reached the end of the footer region.
fn check_footer(popt: Option<&'static dyn Process>,
                checker: &'static dyn AppCredentialsChecker,
                next_footer: usize) -> FooterCheckResult {
    popt.map_or(FooterCheckResult::NoProcess, |process| {
        if config::CONFIG.debug_process_credentials {
            debug!("Checking: Checking {} footer {}", process.get_process_name(), next_footer);
        }
        let footers_position_ptr = process.flash_integrity_end();
        let mut footers_position = footers_position_ptr as usize;
        
        let flash_start_ptr = process.flash_start();
        let flash_start = flash_start_ptr as usize;
        let flash_integrity_len = footers_position - flash_start;
        let flash_end = process.flash_end() as usize;
        let footers_len = flash_end - footers_position;

        if config::CONFIG.debug_process_credentials {
            debug!("Checking: Integrity region is {:x}-{:x}; footers at {:x}-{:x}",
                   flash_start,
                   flash_start + flash_integrity_len,
                   footers_position,
                   flash_end);
        }
        let mut current_footer = 0;
        let mut footer_slice = unsafe {slice::from_raw_parts(footers_position_ptr,
                                                             footers_len)};
        let binary_slice = unsafe {slice::from_raw_parts(flash_start_ptr,
                                                         flash_integrity_len)};
        while current_footer <= next_footer  && footers_position < flash_end {
            if config::CONFIG.debug_process_credentials {
                //debug!("Checking: Checking for footer {}, at {}", next_footer, current_footer);
            }
            let parse_result = tock_tbf::parse::parse_tbf_footer(footer_slice);
            match parse_result {
                Err(TbfParseError::NotEnoughFlash) => {
                    return FooterCheckResult::PastLastFooter;
                }
                Err(TbfParseError::BadTlvEntry(t)) => {
                    if config::CONFIG.debug_process_credentials {
                        debug!("Checking: Bad TLV entry, type: {:?}", t);
                    }
                    return FooterCheckResult::BadFooter;
                }
                Err(e) => {
                    if config::CONFIG.debug_process_credentials {
                        debug!("Checking: Error parsing footer: {:?}", e);
                    }
                    return FooterCheckResult::BadFooter;
                }
                Ok((footer, len)) => {
                    let slice_result = footer_slice.get(len as usize + 4..);
                    footers_position = footers_position + len as usize + 4;
                    match slice_result {
                        None => {
                            return FooterCheckResult::BadFooter;
                        }
                        Some(slice) => {
                            footer_slice = slice;
                            if current_footer == next_footer {
                                match checker.check_credentials(footer, binary_slice) {
                                    Ok(()) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!("Checking: Found {}, checking", current_footer);
                                        }
                                        return FooterCheckResult::Checking;
                                    }
                                    Err((ErrorCode::NOSUPPORT, _, _)) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!("Checking: Found {}, not supported", current_footer);
                                        }
                                        return FooterCheckResult::FooterNotCheckable;
                                    }
                                    Err((ErrorCode::ALREADY, _, _)) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!("Checking: Found {}, already", current_footer);
                                        }
                                        return FooterCheckResult::FooterNotCheckable;
                                    }
                                    Err(e) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!("Checking: Found {}, error {:?}", current_footer, e);
                                        }
                                        return FooterCheckResult::Error;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            current_footer = current_footer + 1;
        }
        FooterCheckResult::PastLastFooter
    })
        
}

impl process_checking::Client<'static> for ProcessCheckerMachine {
    fn check_done(&self,
                  result: Result<process_checking::CheckResult, ErrorCode>,
                  credentials: TbfFooterV2Credentials,
                  _binary: &'static [u8]) {
        let capability = create_capability!(ProcessApprovalCapability);
        if config::CONFIG.debug_process_credentials {
            debug!("Checking: check_done gave result {:?}", result);
        }
        match result {
            Ok(process_checking::CheckResult::Accept) => {
                self.processes[self.process.get()].map(|p| {
                    let _r = p.mark_credentials_pass(Some(credentials), &capability);
                    if p.enqueue_init_task().is_err() {
                        debug!("Error starting checked process {}",
                               p.get_process_name());
                    }
                });
                self.process.set(self.process.get() + 1);
            },
            Ok(process_checking::CheckResult::Pass) => {
                self.footer.set(self.footer.get() + 1);
            },
            Ok(process_checking::CheckResult::Reject) => {
                self.processes[self.process.get()].map(|p| {
                    let _r = p.mark_credentials_fail(&capability);
                });
                self.process.set(self.process.get() + 1);
            }
            Err(e) => {
                if config::CONFIG.debug_process_credentials {
                    debug!("Checking: error checking footer {:?}", e);
                }
                self.footer.set(self.footer.get() + 1);
            }
        }
        let _cont =  self.next();
    }
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
    _capability: &dyn ProcessManagementCapability
) -> Result<(&'static [u8], &'static mut [u8], Option<&'static dyn Process>),
            (&'static [u8], &'static mut [u8], ProcessLoadError)> {
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
        None => return Err((app_flash,
                            app_memory,
                            ProcessLoadError::NotEnoughFlash)),
        Some(val) => val
    };        
    
    // Advance the flash slice for process discovery beyond this last entry.
    // This will be the start of where we look for a new process since Tock
    // processes are allocated back-to-back in flash.
    let remaining_flash = match app_flash.get(entry_flash.len()..) {
        None => return Err((app_flash,
                            app_memory,
                            ProcessLoadError::NotEnoughFlash)),
        Some(val) => val
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
                    process.mem_start() as usize,
                    process.mem_end() as usize - 1,
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
