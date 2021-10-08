//! Interface for RSA Public/Private key encryption math operations

use crate::hil::public_key_crypto::keys::RsaPrivKey;
use crate::ErrorCode;

/// Upcall from the `RsaCrypto` trait.
pub trait Client<'a> {
    /// This callback is called when the mod_exponent operation is complete.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy
    ///    - ALREADY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent_done(
        &'a self,
        status: Result<bool, ErrorCode>,
        message: &'static mut [u8],
        key: &'static mut dyn RsaPrivKey,
        result: &'static mut [u8],
    );
}

pub trait RsaCryptoBase<'a> {
    /// Set the `Client` client to be called on completion.
    fn set_client(&'a self, client: &'a dyn Client<'a>);

    /// Clear any confidential data.
    fn clear_data(&self);

    /// Calculate (message ^ exponent) % modulus and store it in the
    /// `result` buffer. exponent and modulus are supplied from the `key`.
    ///
    /// On completion the `mod_exponent_done()` upcall will be scheduled.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy
    ///    - ALREADY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        key: &'static mut dyn RsaPrivKey,
        result: &'static mut [u8],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static mut dyn RsaPrivKey,
            &'static mut [u8],
        ),
    >;
}
