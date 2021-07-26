//! Helper library for RSA public and private keys

use core::convert::TryInto;
use kernel::hil::public_key_crypto::keys::{PubKey, PubPrivKey, RsaKey, RsaPrivKey};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::mut_imut_buffer::MutImutBuffer;
use kernel::ErrorCode;

/// A Public/Private RSA 2048 key pair
pub struct RSA2048Keys {
    public_key: OptionalCell<MutImutBuffer<'static, u8>>,
    private_key: OptionalCell<MutImutBuffer<'static, u8>>,
}

impl<'a> RSA2048Keys {
    pub const fn new() -> Self {
        Self {
            public_key: OptionalCell::empty(),
            private_key: OptionalCell::empty(),
        }
    }
}

impl PubKey for RSA2048Keys {
    /// `public_key` is a buffer containing the public key.
    /// The first 4 bytes are the public_exponent (also called `e`).
    /// The next 256 bytes is the modulus (also called `n`).
    fn import_public_key(
        &self,
        public_key: MutImutBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)> {
        if public_key.len() - 4 != 256 {
            return Err((ErrorCode::SIZE, public_key));
        }

        self.public_key.replace(public_key);

        Ok(())
    }

    fn pub_key(&self) -> Result<MutImutBuffer<'static, u8>, ErrorCode> {
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

impl PubPrivKey for RSA2048Keys {
    /// `private_key` is a buffer containing the private key.
    /// The first 256 bytes are the private_exponent (also called `d`).
    fn import_private_key(
        &self,
        private_key: MutImutBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)> {
        if private_key.len() != 256 {
            return Err((ErrorCode::SIZE, private_key));
        }

        self.private_key.replace(private_key);

        Ok(())
    }

    fn priv_key(&self) -> Result<MutImutBuffer<'static, u8>, ErrorCode> {
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
            match public_key {
                MutImutBuffer::Mutable(ref buf) => {
                    let _ = closure(&buf[4..]);
                }
                MutImutBuffer::Immutable(buf) => {
                    let _ = closure(&buf[4..]);
                }
            }
            self.public_key.replace(public_key);
            Some(())
        } else {
            None
        }
    }

    fn public_exponent(&self) -> Option<u32> {
        if let Some(key) = self.public_key.take() {
            let ret = Some(u32::from_be_bytes(match key {
                MutImutBuffer::Mutable(ref buf) => buf[0..4].try_into().unwrap(),
                MutImutBuffer::Immutable(ref buf) => buf[0..4].try_into().unwrap(),
            }));
            self.public_key.set(key);
            ret
        } else {
            None
        }
    }
}

impl RsaPrivKey for RSA2048Keys {
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        if let Some(private_key) = self.private_key.take() {
            match private_key {
                MutImutBuffer::Mutable(ref buf) => {
                    let _ = closure(buf);
                }
                MutImutBuffer::Immutable(buf) => {
                    let _ = closure(buf);
                }
            }
            self.private_key.replace(private_key);
            Some(())
        } else {
            None
        }
    }
}
