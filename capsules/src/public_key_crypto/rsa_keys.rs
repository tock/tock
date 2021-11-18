//! Helper library for RSA public and private keys

use core::cell::Cell;
use kernel::hil::public_key_crypto::keys::{PublicKey, PublicPrivateKey, RsaKey, RsaPrivateKey};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

// Copy OpenSSL and use e as 65537
const PUBLIC_EXPONENT: u32 = 65537;

/// A Public/Private RSA 2048 key pair
pub struct RSA2048Keys {
    public_key: OptionalCell<&'static [u8]>,
    public_exponent: Cell<u32>,
    private_key: OptionalCell<&'static [u8]>, 
}

impl<'a> RSA2048Keys {
    pub const fn new() -> Self {
        Self {
            public_key: OptionalCell::empty(),
            public_exponent: Cell::new(PUBLIC_EXPONENT),
            private_key: OptionalCell::empty(),
        }
    }
}

impl PublicKey for RSA2048Keys {
    /// `public_key` is a buffer containing the public key.
    /// This is the 256 byte modulus (also called `n`).
    fn import_public_key(
        &self,
        public_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])> {
        if public_key.len() != 256 {
            return Err((ErrorCode::SIZE, public_key));
        }

        self.public_key.replace(public_key);

        Ok(())
    }

    fn public_key(&self) -> Result<&'static [u8], ErrorCode> {
        if self.public_key.is_some() {
            Ok(self.public_key.take().unwrap())
        } else {
            Err(ErrorCode::NODEVICE)
        }
    }

    fn len(&self) -> usize {
        if let Some(key) = self.public_key.take() {
            let ret = key.len();
            self.public_key.set(key);
            ret
        } else {
            0
        }
    }
}

impl PublicPrivateKey for RSA2048Keys {
    /// `private_key` is a buffer containing the private key.
    /// The first 256 bytes are the private_exponent (also called `d`).
    fn import_private_key(
        &self,
        private_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])> {
        if private_key.len() != 256 {
            return Err((ErrorCode::SIZE, private_key));
        }

        self.private_key.replace(private_key);

        Ok(())
    }

    fn private_key(&self) -> Result<&'static [u8], ErrorCode> {
        if self.private_key.is_some() {
            Ok(self.private_key.take().unwrap())
        } else {
            Err(ErrorCode::NODEVICE)
        }
    }

    fn len(&self) -> usize {
        if let Some(key) = self.private_key.take() {
            let ret = key.len();
            self.private_key.set(key);
            ret
        } else {
            0
        }
    }
}

impl RsaKey for RSA2048Keys {
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        if let Some(public_key) = self.public_key.take() {
            let _ = closure(&public_key[4..]);
            self.public_key.replace(public_key);
            Some(())
        } else {
            None
        }
    }

    fn take_modulus(&self) -> Option<&'static [u8]> {
        if let Some(public_key) = self.public_key.take() {
            Some(public_key)
        } else {
            None
        }
    }

    fn public_exponent(&self) -> Option<u32> {
        Some(self.public_exponent.get())
    }
}

impl RsaPrivateKey for RSA2048Keys {
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        if let Some(private_key) = self.private_key.take() {
            let _ = closure(private_key);
            self.private_key.replace(private_key);
            Some(())
        } else {
            None
        }
    }

    fn take_exponent(&self) -> Option<&'static [u8]> {
        if let Some(private_key) = self.private_key.take() {
            Some(private_key)
        } else {
            None
        }
    }
}
