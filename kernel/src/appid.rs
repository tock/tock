//! Application identifiers, application credentials, and short IDS,
//! used for verifying processes before loading them and imposing
//! access control.
//!
//! ## Application Identifiers Overview
//!
//! TRDXXX describes the design and requirements of application identifiers
//! in detail; interested readers should refer to that document.
//!
//! Application identifiers are how a Tock kernel decides whether to
//! load a process and to apply access policies to processes. There
//! are five major mechanisms:
//!   - Application credentials are data that establishes the identity
//!     of a process binary (e.g., what program it is, who released it).
//!   - Application identifiers are the values defining a process
//!     binary's identity. They are typically generated from application
//!     credentials and can be large (e.g., a 4096-bit RSA key).
//!   - Short IDs are 32-bit application identifiers that are used as
//!     shorthand of application identifiers through the kernel.
//!   - The Verify trait defines an interface to a
//!     policy for what application credentials the kernel accepts and
//!     maps application credentials to appliction identifiers.
//!   - The Compress trait defines an interface for mapping application
//!     identifiers to Short IDs.


/// A call to `Verify::check_credentials` returns whether the
/// credential should be accepted (success, the kernel should stop
/// processing credentials headers), passed (keep on trying to process
/// credentials headers), or rejected (failure, the kernel should stop
/// processing credentials headers).
pub enum VerificationResult {
    Accept,
    Pass,
    Reject
}

/// Implements a policy for checking and approving credentials for a
/// process binary.
pub trait Verify {
    /// Returns what the policy should be if every `TbfHeaderV2Credentials`
    /// header generated a `VerificationResult::Pass`: `true` means the
    /// process binary is rejected, while `false` means it is accepted.
    fn require_credentials(&self) -> bool;

    /// Return whether the Verifier accepts a particular credentials
    /// header. The kernel iterates across credentials headers and
    /// invokes this until it returns `VerificationResult::Accept`,
    /// `VerificationResult::Reject`, or it reaches the last
    /// credentials header.
    fn check_credentials(&self,
                         credentials: &TbfHeaderV2Credentials,
                         binary: &mut [u8]) -> VerificationResult;
}

#[derive(Clone, Copy, Eq)]
struct ShortID {
    id: u32
}

/// Translate the application credentials associated with a process
/// to a `ShortID`, if possible.
pub trait Compress {

    /// Return the `ShortID` for the passed credentials header. If the
    /// credentials do not correspond to any known security group or
    /// privileges, return `None`. A `None` value for a `ShortID` will
    /// fail any access check that requires a `ShortID`.
    fn to_short_id(credentials: &TbfHeaderV2Credentials) -> Option<ShortID>;
}

