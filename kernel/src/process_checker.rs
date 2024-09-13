// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Traits and types for application credentials checkers, used to decide
//! whether an application can be loaded.
//!
//! See the [AppID TRD](../../doc/reference/trd-appid.md).

use core::cell::Cell;
use core::fmt;

use crate::config;
use crate::debug;
use crate::process::Process;
use crate::process::ShortId;
use crate::process_binary::ProcessBinary;
use crate::utilities::cells::{NumericCellExt, OptionalCell};
use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfParseError;

/// Error from checking process credentials.
pub enum ProcessCheckError {
    /// The application checker requires credentials, but the TBF did not
    /// include a credentials that meets the checker's requirements. This can be
    /// either because the TBF has no credentials or the checker policy did not
    /// accept any of the credentials it has.
    CredentialsNotAccepted,

    /// The process contained a credentials which was rejected by the verifier.
    /// The `u32` indicates which credentials was rejected: the first
    /// credentials after the application binary is 0, and each subsequent
    /// credentials increments this counter.
    CredentialsRejected(u32),

    /// Error in the kernel implementation.
    InternalError,
}

impl fmt::Debug for ProcessCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessCheckError::CredentialsNotAccepted => {
                write!(f, "No credentials accepted")
            }

            ProcessCheckError::CredentialsRejected(index) => {
                write!(f, "Credential {} rejected", index)
            }

            ProcessCheckError::InternalError => write!(f, "Error in kernel. Likely a bug."),
        }
    }
}

/// What a AppCredentialsChecker decided a particular application's credential
/// indicates about the runnability of an application binary.
#[derive(Debug)]
pub enum CheckResult {
    /// Accept the credential and run the binary.
    ///
    /// The associated value is an optional opaque usize the credential
    /// checker can return to communication some information about the accepted
    /// credential.
    Accept(Option<CheckResultAcceptMetadata>),
    /// Go to the next credential or in the case of the last one fall
    /// back to the default policy.
    Pass,
    /// Reject the credential and do not run the binary.
    Reject,
}

/// Optional metadata the credential checker can attach to an accepted
/// credential.
///
/// This metadata can be used to provide context for why or how the accepted
/// credential was accepted. For example, this could be set to the index of a
/// public key that was used to verify a cryptographic signature. This value can
/// then be used by the AppId assigner to assign the correct AppId and
/// [`ShortId`].
#[derive(Debug, Copy, Clone)]
pub struct CheckResultAcceptMetadata {
    /// The metadata stored with the accepted credential is a usize that has an
    /// application-specific meaning.
    pub metadata: usize,
}

/// Receives callbacks on whether a credential was accepted or not.
pub trait AppCredentialsPolicyClient<'a> {
    /// The check for a particular credential is complete. Result of the check
    /// is in `result`.
    fn check_done(
        &self,
        result: Result<CheckResult, ErrorCode>,
        credentials: TbfFooterV2Credentials,
        integrity_region: &'a [u8],
    );
}

/// The accepted credential from the credential checker.
///
/// This combines both the credential as stored in the TBF footer with an
/// optional opaque value provided by the checker when it accepted the
/// credential. This value can be used when assigning an AppID to the
/// application based on the how the credential was approved. For example, if
/// the credential checker has a list of valid public keys used to verify
/// signatures, it might set the optional value to the index of the public key
/// in this list.
#[derive(Copy, Clone)]
pub struct AcceptedCredential {
    /// The credential stored in the footer that the credential checker
    /// accepted.
    pub credential: TbfFooterV2Credentials,
    /// An optional opaque value set by the credential checker to store metadata
    /// about the accepted credential. This is credential checker specific.
    pub metadata: Option<CheckResultAcceptMetadata>,
}

/// Implements a Credentials Checking Policy.
pub trait AppCredentialsPolicy<'a> {
    /// Set the client which gets notified after the credential check completes.
    fn set_client(&self, client: &'a dyn AppCredentialsPolicyClient<'a>);

    /// Whether credentials are required or not.
    ///
    /// If this returns `true`, then a process will only be executed if one
    /// credential was accepted. If this returns `false` then a process will be
    /// executed even if no credentials are accepted.
    fn require_credentials(&self) -> bool;

    /// Check a particular credential.
    ///
    /// If credential checking started successfully then this returns `Ok()`.
    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        integrity_region: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}

/// Whether two processes have the same Application Identifier; two
/// processes with the same Application Identifier cannot run concurrently.
pub trait AppUniqueness {
    /// Returns whether `process_a` and `process_b` have a different identifier,
    /// and so can run concurrently. If this returns `false`, the kernel
    /// will not run `process_a` and `process_b` at the same time.
    fn different_identifier(&self, process_a: &ProcessBinary, process_b: &ProcessBinary) -> bool;

    /// Returns whether `process_a` and `process_b` have a different identifier,
    /// and so can run concurrently. If this returns `false`, the kernel
    /// will not run `process_a` and `process_b` at the same time.
    fn different_identifier_process(
        &self,
        process_a: &ProcessBinary,
        process_b: &dyn Process,
    ) -> bool;

    /// Returns whether `process_a` and `process_b` have a different identifier,
    /// and so can run concurrently. If this returns `false`, the kernel
    /// will not run `process_a` and `process_b` at the same time.
    fn different_identifier_processes(
        &self,
        process_a: &dyn Process,
        process_b: &dyn Process,
    ) -> bool;
}

/// Default implementation.
impl AppUniqueness for () {
    fn different_identifier(&self, _process_a: &ProcessBinary, _process_b: &ProcessBinary) -> bool {
        true
    }

    fn different_identifier_process(
        &self,
        _process_a: &ProcessBinary,
        _process_b: &dyn Process,
    ) -> bool {
        true
    }

    fn different_identifier_processes(
        &self,
        _process_a: &dyn Process,
        _process_b: &dyn Process,
    ) -> bool {
        true
    }
}

/// Transforms Application Credentials into a corresponding ShortId.
pub trait Compress {
    /// Create a `ShortId` for `process`.
    ///
    /// If the process was approved to run because of a specific credential, the
    /// `ProcessBinary will have its `credential` filed set to `Some()` with
    /// that credential.
    fn to_short_id(&self, process: &ProcessBinary) -> ShortId;
}

impl Compress for () {
    fn to_short_id(&self, _process: &ProcessBinary) -> ShortId {
        ShortId::LocallyUnique
    }
}

pub trait AppIdPolicy: AppUniqueness + Compress {}
impl<T: AppUniqueness + Compress> AppIdPolicy for T {}

/// Client interface for the outcome of a process credential check.
pub trait ProcessCheckerMachineClient {
    /// Check is finished, and the check result is in `result`.0
    ///
    /// If `result` is `Ok(Option<Credentials>)`, the credentials were either
    /// accepted and the accepted credential is provided, or no credentials were
    /// accepted but none is required.
    ///
    /// If `result` is `Err`, the credentials were not accepted and the policy
    /// denied approving the app.
    fn done(
        &self,
        process_binary: ProcessBinary,
        result: Result<Option<AcceptedCredential>, ProcessCheckError>,
    );
}

/// Outcome from checking a single footer credential.
#[derive(Debug)]
enum FooterCheckResult {
    /// A check has started
    Checking,
    /// There are no more footers, no check started
    PastLastFooter,
    /// The footer isn't a credential, no check started
    FooterNotCheckable,
    /// The footer is invalid, no check started
    BadFooter,
    /// An internal error occurred, no check started
    Error,
}

/// Checks the footers for a `ProcessBinary` and decides whether to continue
/// loading the process based on the checking policy in `checker`.
pub struct ProcessCheckerMachine {
    /// Client for receiving the outcome of the check.
    client: OptionalCell<&'static dyn ProcessCheckerMachineClient>,
    /// Policy for checking credentials.
    policy: OptionalCell<&'static dyn AppCredentialsPolicy<'static>>,
    /// Hold the process binary during checking.
    process_binary: OptionalCell<ProcessBinary>,
    /// Keep track of which footer is being parsed.
    footer_index: Cell<usize>,
}

impl ProcessCheckerMachine {
    pub fn new(policy: &'static dyn AppCredentialsPolicy<'static>) -> Self {
        Self {
            footer_index: Cell::new(0),
            policy: OptionalCell::new(policy),
            process_binary: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static dyn ProcessCheckerMachineClient) {
        self.client.set(client);
    }

    pub fn set_policy(&self, policy: &'static dyn AppCredentialsPolicy<'static>) {
        self.policy.replace(policy);
    }

    /// Check this `process_binary` to see if its credentials are valid.
    ///
    /// This must be called from a interrupt callback chain.
    pub fn check(&self, process_binary: ProcessBinary) -> Result<(), ProcessCheckError> {
        self.footer_index.set(0);
        self.process_binary.set(process_binary);
        self.next()
    }

    /// Must be called from a callback context.
    fn next(&self) -> Result<(), ProcessCheckError> {
        let policy = self.policy.get().ok_or(ProcessCheckError::InternalError)?;
        let pb = self
            .process_binary
            .take()
            .ok_or(ProcessCheckError::InternalError)?;
        let pb_name = pb.header.get_package_name().unwrap_or("");

        // Loop over all footers in the footer region. We don't know how many
        // footers there are, so we use `loop {}`.
        loop {
            let footer_index = self.footer_index.get();

            let check_result = ProcessCheckerMachine::check_footer(&pb, policy, footer_index);

            if config::CONFIG.debug_process_credentials {
                debug!(
                    "Checking: Check status for process {}, footer {}: {:?}",
                    pb_name, footer_index, check_result
                );
            }
            match check_result {
                FooterCheckResult::Checking => {
                    self.process_binary.set(pb);
                    break;
                }
                FooterCheckResult::PastLastFooter | FooterCheckResult::BadFooter => {
                    // We reached the end of the footers without any
                    // credentials or all credentials were Pass: apply
                    // the checker policy to see if the process
                    // should be allowed to run.
                    self.policy.map(|policy| {
                        let requires = policy.require_credentials();

                        let result = if requires {
                            Err(ProcessCheckError::CredentialsNotAccepted)
                        } else {
                            Ok(None)
                        };

                        self.client.map(|client| client.done(pb, result));
                    });
                    break;
                }
                FooterCheckResult::FooterNotCheckable => {
                    // Go to next footer
                    self.footer_index.increment();
                }
                FooterCheckResult::Error => {
                    self.client
                        .map(|client| client.done(pb, Err(ProcessCheckError::InternalError)));
                    break;
                }
            }
        }
        Ok(())
    }

    // Returns whether a footer is being checked or not, and if not, why.
    // Iterates through the footer list until if finds `next_footer` or
    // it reached the end of the footer region.
    fn check_footer(
        process_binary: &ProcessBinary,
        policy: &'static dyn AppCredentialsPolicy<'static>,
        next_footer: usize,
    ) -> FooterCheckResult {
        if config::CONFIG.debug_process_credentials {
            debug!(
                "Checking: Checking {:?} footer {}",
                process_binary.header.get_package_name(),
                next_footer
            );
        }

        let integrity_slice = process_binary.get_integrity_region_slice();
        let mut footer_slice = process_binary.footers;

        if config::CONFIG.debug_process_credentials {
            debug!(
                "Checking: Integrity region is {:x}-{:x}; footers at {:x}-{:x}",
                integrity_slice.as_ptr() as usize,
                integrity_slice.as_ptr() as usize + integrity_slice.len(),
                footer_slice.as_ptr() as usize,
                footer_slice.as_ptr() as usize + footer_slice.len(),
            );
        }

        let mut current_footer = 0;
        while current_footer <= next_footer {
            if config::CONFIG.debug_process_credentials {
                debug!(
                    "Checking: Current footer slice {:x}-{:x}",
                    footer_slice.as_ptr() as usize,
                    footer_slice.as_ptr() as usize + footer_slice.len(),
                );
            }

            let parse_result = tock_tbf::parse::parse_tbf_footer(footer_slice);
            match parse_result {
                Err(TbfParseError::NotEnoughFlash) => {
                    if config::CONFIG.debug_process_credentials {
                        debug!("Checking: Not enough flash for a footer");
                    }
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
                    if config::CONFIG.debug_process_credentials {
                        debug!(
                            "ProcessCheck: @{:x} found a len {} footer: {:?}",
                            footer_slice.as_ptr() as usize,
                            len,
                            footer.format()
                        );
                    }
                    match slice_result {
                        None => {
                            return FooterCheckResult::BadFooter;
                        }
                        Some(slice) => {
                            footer_slice = slice;
                            if current_footer == next_footer {
                                match policy.check_credentials(footer, integrity_slice) {
                                    Ok(()) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!("Checking: Found {}, checking", current_footer);
                                        }
                                        return FooterCheckResult::Checking;
                                    }
                                    Err((ErrorCode::NOSUPPORT, _, _)) => {
                                        if config::CONFIG.debug_process_credentials {
                                            debug!(
                                                "Checking: Found {}, not supported",
                                                current_footer
                                            );
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
                                            debug!(
                                                "Checking: Found {}, error {:?}",
                                                current_footer, e
                                            );
                                        }
                                        return FooterCheckResult::Error;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            current_footer += 1;
        }
        FooterCheckResult::PastLastFooter
    }
}

impl AppCredentialsPolicyClient<'static> for ProcessCheckerMachine {
    fn check_done(
        &self,
        result: Result<CheckResult, ErrorCode>,
        credentials: TbfFooterV2Credentials,
        _integrity_region: &'static [u8],
    ) {
        if config::CONFIG.debug_process_credentials {
            debug!("Checking: check_done gave result {:?}", result);
        }
        let cont = match result {
            Ok(CheckResult::Accept(opaque)) => {
                self.client.map(|client| {
                    if let Some(pb) = self.process_binary.take() {
                        client.done(
                            pb,
                            Ok(Some(AcceptedCredential {
                                credential: credentials,
                                metadata: opaque,
                            })),
                        )
                    }
                });
                false
            }
            Ok(CheckResult::Pass) => {
                // Checker ignored the credential, so we try the next one.
                self.footer_index.increment();
                true
            }
            Ok(CheckResult::Reject) => {
                self.client.map(|client| {
                    if let Some(pb) = self.process_binary.take() {
                        client.done(
                            pb,
                            Err(ProcessCheckError::CredentialsRejected(
                                self.footer_index.get() as u32,
                            )),
                        )
                    }
                });
                false
            }
            Err(e) => {
                if config::CONFIG.debug_process_credentials {
                    debug!("Checking: error checking footer {:?}", e);
                }
                self.footer_index.increment();
                true
            }
        };
        if cont {
            // If this errors it is an internal error. We don't have a
            // `process_binary` to signal the `client::done()` callback, so we
            // cannot signal the error.
            let _ = self.next();
        }
    }
}
