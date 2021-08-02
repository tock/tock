//! Helper library for RSA public and private keys

use core::cell::Cell;
use kernel::hil::entropy;
use kernel::hil::public_key_crypto::{PubPrivKey, PubPrivKeyClient, RsaKey};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct RSA2048Keys<'a> {
    modulus: [u8; 256],          // Also called n
    public_exponent: u32,        // Also called e
    private_exponent: [u8; 256], // Also called d
    client: OptionalCell<&'a dyn PubPrivKeyClient<'a>>,
    pub_initialised: Cell<bool>,
    priv_initialised: Cell<bool>,
}

impl<'a> PubPrivKeyClient<'a> for RSA2048Keys<'a> {
    fn generation_complete(&'a self, result: Result<(), ErrorCode>) {
        if result.is_ok() {
            self.pub_initialised.set(true);
            self.priv_initialised.set(true);
        }
    }
}

impl<'a> RSA2048Keys<'a> {
    pub const fn new() -> Self {
        Self {
            modulus: [0; 256],
            // Use the same default as OpenSSL
            public_exponent: 0x10001,
            private_exponent: [0; 256],
            client: OptionalCell::empty(),
            pub_initialised: Cell::new(false),
            priv_initialised: Cell::new(false),
        }
    }
}

impl<'a> PubPrivKey<'a> for RSA2048Keys<'a> {
    fn set_client(&'a self, client: &'a dyn PubPrivKeyClient<'a>) {
        self.client.set(client);
    }

    fn generate(
        &'a self,
        _trng: &'a dyn entropy::Entropy32,
        _length: usize,
    ) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /// `public_key` is a buffer containing the public key.
    /// The first 4 bytes are the public_exponent (also called `e`).
    fn import_public_key(&mut self, public_key: &[u8]) -> Result<(), ErrorCode> {
        if public_key.len() - 4 != self.modulus.len() {
            return Err(ErrorCode::SIZE);
        }

        self.public_exponent = public_key[0] as u32
            | (public_key[1] as u32) << 8
            | (public_key[2] as u32) << 16
            | (public_key[3] as u32) << 24;
        self.modulus.copy_from_slice(&public_key[4..]);
        self.pub_initialised.set(true);

        Ok(())
    }

    /// `private_key` is a buffer containing the private key.
    /// The first 256 bytes are the private_exponent (also called `d`).
    fn import_private_key(&mut self, private_key: &[u8]) -> Result<(), ErrorCode> {
        if private_key.len() != self.private_exponent.len() + self.modulus.len() {
            return Err(ErrorCode::SIZE);
        }

        let private_exponent_len = self.private_exponent.len();

        self.private_exponent
            .copy_from_slice(&private_key[0..private_exponent_len]);
        self.modulus
            .copy_from_slice(&private_key[private_exponent_len..]);
        self.priv_initialised.set(true);

        Ok(())
    }

    fn pub_key(&'a self, buffer: &mut [u8]) -> Option<()> {
        if self.pub_initialised.get() {
            buffer[..4].copy_from_slice(&self.public_exponent.to_ne_bytes());
            buffer[4..].copy_from_slice(&self.modulus);
            Some(())
        } else {
            None
        }
    }

    fn priv_key(&'a self, buffer: &mut [u8]) -> Option<()> {
        if self.priv_initialised.get() {
            let private_exponent_len = self.private_exponent.len();

            buffer[..private_exponent_len].copy_from_slice(&self.private_exponent);
            buffer[private_exponent_len..].copy_from_slice(&self.modulus);

            Some(())
        } else {
            None
        }
    }
}

impl<'a> RsaKey<'a> for RSA2048Keys<'a> {
    fn modulus(&'a self) -> Option<&'a [u8]> {
        if self.pub_initialised.get() || self.priv_initialised.get() {
            Some(&self.modulus)
        } else {
            None
        }
    }

    fn public_exponent(&'a self) -> Option<u32> {
        if self.pub_initialised.get() {
            Some(self.public_exponent)
        } else {
            None
        }
    }

    fn private_exponent(&'a self) -> Option<&'a [u8]> {
        if self.priv_initialised.get() {
            Some(&self.private_exponent)
        } else {
            None
        }
    }
}
