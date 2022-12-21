//! Traits and types for application credentials checkers, used to
//! decide whether an application can be loaded. See
//| the [AppID TRD](../../doc/reference/trd-appid.md).

pub mod basic;

use crate::process::{Process, ShortID, State};
use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;

/// What a AppCredentialsChecker decided a particular application's credential
/// indicates about the runnability of an application binary.
#[derive(Debug)]
pub enum CheckResult {
    /// Accept the credential and run the binary.
    Accept,
    /// Go to the next credential or in the case of the last one fall
    /// back to the default policy.
    Pass,
    /// Reject the credential and do not run the binary.
    Reject,
}

/// Receives callbacks on whether a credential was accepted or not.
pub trait Client<'a> {
    fn check_done(
        &self,
        result: Result<CheckResult, ErrorCode>,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    );
}

/// Implements a Credentials Checking Policy.
pub trait AppCredentialsChecker<'a> {
    fn set_client(&self, _client: &'a dyn Client<'a>);
    fn require_credentials(&self) -> bool;
    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}

/// Default implementation.
impl<'a> AppCredentialsChecker<'a> for () {
    fn set_client(&self, _client: &'a dyn Client<'a>) {}
    fn require_credentials(&self) -> bool {
        false
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        Err((ErrorCode::NOSUPPORT, credentials, binary))
    }
}

/// Return whether there is a currently running process that has
/// the same application identifier as `process` OR the same short
/// ID as `process`. This means that if `process` is currently
/// running, `has_unique_identifier` returns false.
pub fn has_unique_identifiers<AU: AppUniqueness>(
    process: &dyn Process,
    processes: &[Option<&dyn Process>],
    id_differ: &AU,
) -> bool {
    let len = processes.len();
    // If the process is running or not runnable it does not have
    // a unique identifier; these two states describe a process
    // that is potentially runnable, dependent on checking for
    // identifier uniqueness at runtime.
    if process.get_state() != State::CredentialsApproved && process.get_state() != State::Terminated
    {
        return false;
    }

    // Note that this causes `process` to compare against itself;
    // however, since `process` should not be running, it will
    // not check the identifiers and say they are different. This means
    // this method returns false if the process is running.
    for i in 0..len {
        let checked_process = processes[i];
        let diff = checked_process.map_or(true, |other| {
            !other.is_running()
                || (id_differ.different_identifier(process, other)
                    && other.short_app_id() != process.short_app_id())
        });
        if !diff {
            return false;
        }
    }
    true
}

/// Whether two processes have the same Application Identifier; two
/// processes with the same Application Identifier cannot run concurrently.
pub trait AppUniqueness {
    /// Returns whether `process_a` and `process_b` have a different identifier,
    /// and so can run concurrently. If this returns `false`, the kernel
    /// will not run `process_a` and `process_b` at the same time.
    fn different_identifier(&self, _process_a: &dyn Process, _process_b: &dyn Process) -> bool;
}

/// Default implementation.
impl AppUniqueness for () {
    fn different_identifier(&self, _process_a: &dyn Process, _process_b: &dyn Process) -> bool {
        true
    }
}

/// Transforms Application Credentials into a corresponding ShortID.
pub trait Compress {
    fn to_short_id(&self, _credentials: &TbfFooterV2Credentials) -> ShortID;
}

impl Compress for () {
    fn to_short_id(&self, _credentials: &TbfFooterV2Credentials) -> ShortID {
        ShortID::LocallyUnique
    }
}

pub trait CredentialsCheckingPolicy<'a>:
    AppCredentialsChecker<'a> + Compress + AppUniqueness
{
}
impl<'a, T: AppCredentialsChecker<'a> + Compress + AppUniqueness> CredentialsCheckingPolicy<'a>
    for T
{
}
