// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Representation of processes stored in flash.
//!
//! A `ProcessBinary` object represents the stored binary for a process before
//! it is loaded into a runnable `Process` object.

use core::fmt;

use crate::config;
use crate::debug;
use crate::process_checker::AcceptedCredential;
use crate::utilities::cells::OptionalCell;

/// Errors resulting from trying to load a process binary structure from flash.
pub enum ProcessBinaryError {
    /// No TBF header was found.
    TbfHeaderNotFound,

    /// The TBF header for the process could not be successfully parsed.
    TbfHeaderParseFailure(tock_tbf::types::TbfParseError),

    /// Not enough flash remaining to parse a process and its header.
    NotEnoughFlash,

    /// A process requires a newer version of the kernel or did not specify a
    /// required version. Processes can include the KernelVersion TBF header
    /// stating their compatible kernel version (^major.minor).
    ///
    /// Boards may not require processes to include the KernelVersion TBF
    /// header, and the kernel supports ignoring a missing KernelVersion TBF
    /// header. In that case, this error will not be returned for a process
    /// missing a KernelVersion TBF header.
    ///
    /// `version` is the `(major, minor)` kernel version the process indicates
    /// it requires. If `version` is `None` then the process did not include the
    /// KernelVersion TBF header.
    IncompatibleKernelVersion { version: Option<(u16, u16)> },

    /// A process specified that its binary must start at a particular address,
    /// and that is not the address the binary is actually placed at.
    IncorrectFlashAddress {
        actual_address: u32,
        expected_address: u32,
    },

    /// The process binary specifies the process is not enabled, and therefore
    /// cannot be loaded.
    NotEnabledProcess,

    /// This entry in flash is just padding.
    Padding,
}

impl From<tock_tbf::types::TbfParseError> for ProcessBinaryError {
    /// Convert between a TBF Header parse error and a process binary error.
    ///
    /// We note that the process binary error is because a TBF header failed to
    /// parse, and just pass through the parse error.
    fn from(error: tock_tbf::types::TbfParseError) -> Self {
        ProcessBinaryError::TbfHeaderParseFailure(error)
    }
}

impl fmt::Debug for ProcessBinaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessBinaryError::TbfHeaderNotFound => {
                write!(f, "Could not find TBF header")
            }

            ProcessBinaryError::TbfHeaderParseFailure(tbf_parse_error) => {
                writeln!(f, "Error parsing TBF header")?;
                write!(f, "{:?}", tbf_parse_error)
            }

            ProcessBinaryError::NotEnoughFlash => {
                write!(f, "Not enough flash available for TBF")
            }

            ProcessBinaryError::IncorrectFlashAddress {
                actual_address,
                expected_address,
            } => write!(
                f,
                "App flash does not match requested address. Actual:{:#x}, Expected:{:#x}",
                actual_address, expected_address
            ),

            ProcessBinaryError::IncompatibleKernelVersion { version } => match version {
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

            ProcessBinaryError::NotEnabledProcess => {
                write!(f, "Process marked not enabled")
            }

            ProcessBinaryError::Padding => {
                write!(f, "Process item is just padding")
            }
        }
    }
}

/// A process stored in flash.
pub struct ProcessBinary {
    /// Process flash segment. This is the entire region of nonvolatile flash
    /// that the process occupies.
    pub flash: &'static [u8],

    /// The footers of the process binary (may be zero-sized), which are metadata
    /// about the process not covered by integrity. Used, among other things, to
    /// store signatures.
    pub footers: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    pub header: tock_tbf::types::TbfHeader<'static>,

    /// Optional credential that was used to approve this application. This is
    /// set if the process is checked by a credential checker and a specific
    /// credential was used to approve this process. Otherwise this is `None`.
    pub credential: OptionalCell<AcceptedCredential>,
}

impl ProcessBinary {
    pub(crate) fn create(
        app_flash: &'static [u8],
        header_length: usize,
        tbf_version: u16,
        require_kernel_version: bool,
    ) -> Result<Self, ProcessBinaryError> {
        // Get a slice for just the app header.
        let header_flash = app_flash
            .get(0..header_length)
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;

        // Parse the full TBF header to see if this is a valid app. If the
        // header can't parse, we will error right here.
        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, tbf_version)?;

        // If this isn't an app (i.e. it is padding) then we can skip it and do
        // not create a `ProcessBinary` object.
        if !tbf_header.is_app() {
            if config::CONFIG.debug_load_processes && !tbf_header.is_app() {
                debug!(
                    "Padding in flash={:#010X}-{:#010X}",
                    app_flash.as_ptr() as usize,
                    app_flash.as_ptr() as usize + app_flash.len() - 1
                );
            }
            // Return no process and the full memory slice we were given.
            return Err(ProcessBinaryError::Padding);
        }

        // If this is an app but it isn't enabled, then we can return an error.
        if !tbf_header.enabled() {
            if config::CONFIG.debug_load_processes {
                debug!(
                    "Process not enabled flash={:#010X}-{:#010X} process={:?}",
                    app_flash.as_ptr() as usize,
                    app_flash.as_ptr() as usize + app_flash.len() - 1,
                    tbf_header.get_package_name().unwrap_or("(no name)")
                );
            }
            return Err(ProcessBinaryError::NotEnabledProcess);
        }

        if let Some((major, minor)) = tbf_header.get_kernel_version() {
            // If the `KernelVersion` header is present, we read the requested
            // kernel version and compare it to the running kernel version.
            if crate::KERNEL_MAJOR_VERSION != major || crate::KERNEL_MINOR_VERSION < minor {
                // If the kernel major version is different, we prevent the
                // process from being loaded.
                //
                // If the kernel major version is the same, we compare the
                // kernel minor version. The current running kernel minor
                // version has to be greater or equal to the one that the
                // process has requested. If not, we prevent the process from
                // loading.
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "WARN process {} requires kernel>={}.{} and <{}.0, (running kernel {}.{})",
                        tbf_header.get_package_name().unwrap_or(""),
                        major,
                        minor,
                        (major + 1),
                        crate::KERNEL_MAJOR_VERSION,
                        crate::KERNEL_MINOR_VERSION
                    );
                }
                return Err(ProcessBinaryError::IncompatibleKernelVersion {
                    version: Some((major, minor)),
                });
            }
        } else if require_kernel_version {
            // If enforcing the kernel version is requested, and the
            // `KernelVersion` header is not present, we prevent the process
            // from loading.
            if config::CONFIG.debug_load_processes {
                debug!(
                    "WARN process {} has no kernel version header",
                    tbf_header.get_package_name().unwrap_or("")
                );
                debug!("Please upgrade to elf2tab >= 0.8.0");
            }
            return Err(ProcessBinaryError::IncompatibleKernelVersion { version: None });
        }

        let binary_end = tbf_header.get_binary_end() as usize;
        let total_size = app_flash.len();

        // End of the portion of the application binary covered by integrity.
        // Now handle footers.
        let footer_region = app_flash
            .get(binary_end..total_size)
            .ok_or(ProcessBinaryError::NotEnoughFlash)?;

        // Check that the process is at the correct location in flash if the TBF
        // header specified a fixed address. If there is a mismatch we catch
        // that early.
        if let Some(fixed_flash_start) = tbf_header.get_fixed_address_flash() {
            // The flash address in the header is based on the app binary, so we
            // need to take into account the header length.
            let actual_address = app_flash.as_ptr() as u32 + tbf_header.get_protected_size();
            let expected_address = fixed_flash_start;
            if actual_address != expected_address {
                return Err(ProcessBinaryError::IncorrectFlashAddress {
                    actual_address,
                    expected_address,
                });
            }
        }

        Ok(Self {
            header: tbf_header,
            footers: footer_region,
            flash: app_flash,
            credential: OptionalCell::empty(),
        })
    }

    pub fn get_credential(&self) -> Option<AcceptedCredential> {
        self.credential.get()
    }

    pub(crate) fn get_integrity_region_slice(&self) -> &'static [u8] {
        unsafe {
            core::slice::from_raw_parts(self.flash.as_ptr(), self.header.get_binary_end() as usize)
        }
    }
}
