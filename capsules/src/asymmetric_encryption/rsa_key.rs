//! Helper library for RSA public and private keys

use core::cell::Cell;
use kernel::hil::asymmetric_encryption::{PubPrivKey, PubPrivKeyClient};
use kernel::hil::entropy;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct RSA2048Keys<'a> {
    public: [u8; 256],
    pub private: [u8; 256],
    client: OptionalCell<&'a dyn PubPrivKeyClient<'a>>,
    initialised: Cell<bool>,
}

impl<'a> PubPrivKeyClient<'a> for RSA2048Keys<'a> {
    fn generation_complete(&'a self, result: Result<(), ErrorCode>) {
        if result.is_ok() {
            self.initialised.set(true);
        }
    }
}

impl<'a> RSA2048Keys<'a> {
    pub const fn new() -> Self {
        Self {
            public: [0; 256],
            private: [0; 256],
            client: OptionalCell::empty(),
            initialised: Cell::new(false),
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

    fn import_public_key(&mut self, public_key: &[u8]) -> Result<(), ErrorCode> {
        if public_key.len() != self.public.len() {
            return Err(ErrorCode::SIZE);
        }

        self.public.copy_from_slice(public_key);
        self.initialised.set(true);

        Ok(())
    }

    fn import_private_key(&mut self, private_key: &[u8]) -> Result<(), ErrorCode> {
        if private_key.len() != self.private.len() {
            return Err(ErrorCode::SIZE);
        }

        self.private.copy_from_slice(private_key);
        self.initialised.set(true);

        Ok(())
    }

    fn pub_key(&'a self) -> Option<&'a [u8]> {
        if self.initialised.get() {
            Some(&self.public)
        } else {
            None
        }
    }

    fn priv_key(&'a self) -> Option<&'a [u8]> {
        if self.initialised.get() {
            Some(&self.private)
        } else {
            None
        }
    }
}
