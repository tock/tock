//! Helper library for RSA public and private keys

use core::cell::Cell;
use kernel::hil::public_key_crypto::keys::{
    PubKey, PubKeyMut, PubPrivKey, PubPrivKeyMut, RsaKey, RsaKeyMut, RsaPrivKey, RsaPrivKeyMut,
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::mut_imut_buffer::MutImutBuffer;
use kernel::ErrorCode;

// Copy OpenSSL and use e as 65537
const PUBLIC_EXPONENT: u32 = 65537;

/// A Public/Private RSA key pair
/// The key is `L` bytes long
struct RSAKeys<const L: usize> {
    public_key: OptionalCell<MutImutBuffer<'static, u8>>,
    public_exponent: Cell<u32>,
    private_key: OptionalCell<MutImutBuffer<'static, u8>>,
}

impl<'a, const L: usize> RSAKeys<L> {
    const fn new() -> Self {
        Self {
            public_key: OptionalCell::empty(),
            public_exponent: Cell::new(PUBLIC_EXPONENT),
            private_key: OptionalCell::empty(),
        }
    }

    /// `public_key` is a buffer containing the public key.
    /// This is the `L` byte modulus (also called `n`).
    fn import_public_key(
        &self,
        public_key: MutImutBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)> {
        if public_key.len() != L {
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

    fn import_private_key(
        &self,
        private_key: MutImutBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, MutImutBuffer<'static, u8>)> {
        if private_key.len() != L {
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
}

impl<const L: usize> PubKey for RSAKeys<L> {
    /// `public_key` is a buffer containing the public key.
    /// This is the `L` byte modulus (also called `n`).
    fn import_public_key(
        &self,
        public_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])> {
        if public_key.len() != L {
            return Err((ErrorCode::SIZE, public_key));
        }

        self.public_key
            .replace(MutImutBuffer::Immutable(public_key));

        Ok(())
    }

    fn pub_key(&self) -> Result<&'static [u8], kernel::ErrorCode> {
        if self.public_key.is_some() {
            match self.public_key.take().unwrap() {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            }
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

impl<const L: usize> PubKeyMut for RSAKeys<L> {
    /// `public_key` is a buffer containing the public key.
    /// This is the `L` byte modulus (also called `n`).
    fn import_public_key(
        &self,
        public_key: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if public_key.len() != L {
            return Err((ErrorCode::SIZE, public_key));
        }

        self.public_key.replace(MutImutBuffer::Mutable(public_key));

        Ok(())
    }

    fn pub_key(&self) -> Result<&'static mut [u8], kernel::ErrorCode> {
        if self.public_key.is_some() {
            match self.public_key.take().unwrap() {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            }
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

impl<const L: usize> PubPrivKey for RSAKeys<L> {
    /// `private_key` is a buffer containing the private key.
    /// The first `L` bytes are the private_exponent (also called `d`).
    fn import_private_key(
        &self,
        private_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])> {
        if private_key.len() != L {
            return Err((ErrorCode::SIZE, private_key));
        }

        self.private_key
            .replace(MutImutBuffer::Immutable(private_key));

        Ok(())
    }

    fn priv_key(&self) -> Result<&'static [u8], ErrorCode> {
        if self.private_key.is_some() {
            match self.private_key.take().unwrap() {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            }
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

impl<const L: usize> PubPrivKeyMut for RSAKeys<L> {
    /// `private_key` is a buffer containing the private key.
    /// The first `L` bytes are the private_exponent (also called `d`).
    fn import_private_key(
        &self,
        private_key: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if private_key.len() != L {
            return Err((ErrorCode::SIZE, private_key));
        }

        self.private_key
            .replace(MutImutBuffer::Mutable(private_key));

        Ok(())
    }

    fn priv_key(&self) -> Result<&'static mut [u8], ErrorCode> {
        if self.private_key.is_some() {
            match self.private_key.take().unwrap() {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            }
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

impl<const L: usize> RsaKey for RSAKeys<L> {
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        if let Some(public_key) = self.public_key.take() {
            match public_key {
                MutImutBuffer::Mutable(ref _buf) => unreachable!(),
                MutImutBuffer::Immutable(buf) => {
                    let _ = closure(buf);
                }
            }
            self.public_key.replace(public_key);
            Some(())
        } else {
            None
        }
    }

    fn take_modulus(&self) -> Option<&'static [u8]> {
        if let Some(public_key) = self.public_key.take() {
            match public_key {
                MutImutBuffer::Immutable(ret) => Some(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            }
        } else {
            None
        }
    }

    fn public_exponent(&self) -> Option<u32> {
        Some(self.public_exponent.get())
    }
}

impl<const L: usize> RsaKeyMut for RSAKeys<L> {
    fn map_modulus(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        if let Some(mut public_key) = self.public_key.take() {
            match public_key {
                MutImutBuffer::Mutable(ref mut buf) => {
                    let _ = closure(buf);
                }
                MutImutBuffer::Immutable(_buf) => unreachable!(),
            }
            self.public_key.replace(public_key);
            Some(())
        } else {
            None
        }
    }

    fn take_modulus(&self) -> Option<&'static mut [u8]> {
        if let Some(public_key) = self.public_key.take() {
            match public_key {
                MutImutBuffer::Mutable(ret) => Some(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            }
        } else {
            None
        }
    }

    fn public_exponent(&self) -> Option<u32> {
        Some(self.public_exponent.get())
    }
}

impl<const L: usize> RsaPrivKey for RSAKeys<L> {
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        if let Some(private_key) = self.private_key.take() {
            match private_key {
                MutImutBuffer::Mutable(ref _buf) => unreachable!(),
                MutImutBuffer::Immutable(buf) => {
                    let _ = closure(buf);
                }
            };
            self.private_key.replace(private_key);
            Some(())
        } else {
            None
        }
    }

    fn take_exponent(&self) -> Option<&'static [u8]> {
        if let Some(private_key) = self.private_key.take() {
            match private_key {
                MutImutBuffer::Immutable(ret) => Some(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            }
        } else {
            None
        }
    }
}

impl<const L: usize> RsaPrivKeyMut for RSAKeys<L> {
    fn map_exponent(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        if let Some(mut private_key) = self.private_key.take() {
            match private_key {
                MutImutBuffer::Mutable(ref mut buf) => {
                    let _ = closure(buf);
                }
                MutImutBuffer::Immutable(_buf) => unreachable!(),
            };
            self.private_key.replace(private_key);
            Some(())
        } else {
            None
        }
    }

    fn take_exponent(&self) -> Option<&'static mut [u8]> {
        if let Some(private_key) = self.private_key.take() {
            match private_key {
                MutImutBuffer::Mutable(ret) => Some(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            }
        } else {
            None
        }
    }
}

pub struct RSA2048Keys(RSAKeys<256>);

impl RSA2048Keys {
    pub const fn new() -> RSA2048Keys {
        RSA2048Keys(RSAKeys::<256>::new())
    }
}

impl PubKey for RSA2048Keys {
    fn import_public_key(
        &self,
        public_key: &'static [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static [u8])> {
        let key = self
            .0
            .import_public_key(MutImutBuffer::Immutable(public_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Immutable(ret) => Err((e, ret)),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn pub_key(&self) -> Result<&'static [u8], kernel::ErrorCode> {
        match self.0.pub_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubKey::len(&self.0)
    }
}

impl PubPrivKey for RSA2048Keys {
    fn import_private_key(
        &self,
        private_key: &'static [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static [u8])> {
        let key = self
            .0
            .import_private_key(MutImutBuffer::Immutable(private_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Immutable(ret) => Err((e, ret)),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn priv_key(&self) -> Result<&'static [u8], kernel::ErrorCode> {
        match self.0.priv_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubPrivKey::len(&self.0)
    }
}

impl RsaKey for RSA2048Keys {
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        RsaKey::map_modulus(&self.0, closure)
    }

    fn take_modulus(&self) -> Option<&'static [u8]> {
        RsaKey::take_modulus(&self.0)
    }

    fn public_exponent(&self) -> Option<u32> {
        RsaKey::public_exponent(&self.0)
    }
}

impl RsaPrivKey for RSA2048Keys {
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        RsaPrivKey::map_exponent(&self.0, closure)
    }

    fn take_exponent(&self) -> Option<&'static [u8]> {
        RsaPrivKey::take_exponent(&self.0)
    }
}

pub struct RSA2048KeysMut(RSAKeys<256>);

impl RSA2048KeysMut {
    pub const fn new() -> RSA2048KeysMut {
        RSA2048KeysMut(RSAKeys::<256>::new())
    }
}

impl PubKeyMut for RSA2048KeysMut {
    fn import_public_key(
        &self,
        public_key: &'static mut [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        let key = self.0.import_public_key(MutImutBuffer::Mutable(public_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Mutable(ret) => Err((e, ret)),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn pub_key(&self) -> Result<&'static mut [u8], kernel::ErrorCode> {
        match self.0.pub_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubKey::len(&self.0)
    }
}

impl PubPrivKeyMut for RSA2048KeysMut {
    fn import_private_key(
        &self,
        private_key: &'static mut [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        let key = self
            .0
            .import_private_key(MutImutBuffer::Mutable(private_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Mutable(ret) => Err((e, ret)),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn priv_key(&self) -> Result<&'static mut [u8], kernel::ErrorCode> {
        match self.0.priv_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubPrivKey::len(&self.0)
    }
}

impl RsaKeyMut for RSA2048KeysMut {
    fn map_modulus(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        RsaKeyMut::map_modulus(&self.0, closure)
    }

    fn take_modulus(&self) -> Option<&'static mut [u8]> {
        RsaKeyMut::take_modulus(&self.0)
    }

    fn public_exponent(&self) -> Option<u32> {
        RsaKeyMut::public_exponent(&self.0)
    }
}

impl RsaPrivKeyMut for RSA2048KeysMut {
    fn map_exponent(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        RsaPrivKeyMut::map_exponent(&self.0, closure)
    }

    fn take_exponent(&self) -> Option<&'static mut [u8]> {
        RsaPrivKeyMut::take_exponent(&self.0)
    }
}

pub struct RSA4096Keys(RSAKeys<512>);

impl RSA4096Keys {
    pub const fn new() -> RSA4096Keys {
        RSA4096Keys(RSAKeys::<512>::new())
    }
}

impl PubKey for RSA4096Keys {
    fn import_public_key(
        &self,
        public_key: &'static [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static [u8])> {
        let key = self
            .0
            .import_public_key(MutImutBuffer::Immutable(public_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Immutable(ret) => Err((e, ret)),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn pub_key(&self) -> Result<&'static [u8], kernel::ErrorCode> {
        match self.0.pub_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubKey::len(&self.0)
    }
}

impl PubPrivKey for RSA4096Keys {
    fn import_private_key(
        &self,
        private_key: &'static [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static [u8])> {
        let key = self
            .0
            .import_private_key(MutImutBuffer::Immutable(private_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Immutable(ret) => Err((e, ret)),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn priv_key(&self) -> Result<&'static [u8], kernel::ErrorCode> {
        match self.0.priv_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Immutable(ret) => Ok(ret),
                MutImutBuffer::Mutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubPrivKey::len(&self.0)
    }
}

impl RsaKey for RSA4096Keys {
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        RsaKey::map_modulus(&self.0, closure)
    }

    fn take_modulus(&self) -> Option<&'static [u8]> {
        RsaKey::take_modulus(&self.0)
    }

    fn public_exponent(&self) -> Option<u32> {
        RsaKey::public_exponent(&self.0)
    }
}

impl RsaPrivKey for RSA4096Keys {
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()> {
        RsaPrivKey::map_exponent(&self.0, closure)
    }

    fn take_exponent(&self) -> Option<&'static [u8]> {
        RsaPrivKey::take_exponent(&self.0)
    }
}

pub struct RSA4096KeysMut(RSAKeys<512>);

impl RSA4096KeysMut {
    pub const fn new() -> RSA4096KeysMut {
        RSA4096KeysMut(RSAKeys::<512>::new())
    }
}

impl PubKeyMut for RSA4096KeysMut {
    fn import_public_key(
        &self,
        public_key: &'static mut [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        let key = self.0.import_public_key(MutImutBuffer::Mutable(public_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Mutable(ret) => Err((e, ret)),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn pub_key(&self) -> Result<&'static mut [u8], kernel::ErrorCode> {
        match self.0.pub_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubKey::len(&self.0)
    }
}

impl PubPrivKeyMut for RSA4096KeysMut {
    fn import_private_key(
        &self,
        private_key: &'static mut [u8],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        let key = self
            .0
            .import_private_key(MutImutBuffer::Mutable(private_key));

        match key {
            Err((e, buf)) => match buf {
                MutImutBuffer::Mutable(ret) => Err((e, ret)),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Ok(()) => Ok(()),
        }
    }

    fn priv_key(&self) -> Result<&'static mut [u8], kernel::ErrorCode> {
        match self.0.priv_key() {
            Ok(buf) => match buf {
                MutImutBuffer::Mutable(ret) => Ok(ret),
                MutImutBuffer::Immutable(_ret) => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }

    fn len(&self) -> usize {
        PubPrivKey::len(&self.0)
    }
}

impl RsaKeyMut for RSA4096KeysMut {
    fn map_modulus(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        RsaKeyMut::map_modulus(&self.0, closure)
    }

    fn take_modulus(&self) -> Option<&'static mut [u8]> {
        RsaKeyMut::take_modulus(&self.0)
    }

    fn public_exponent(&self) -> Option<u32> {
        RsaKeyMut::public_exponent(&self.0)
    }
}

impl RsaPrivKeyMut for RSA4096KeysMut {
    fn map_exponent(&self, closure: &dyn Fn(&mut [u8]) -> ()) -> Option<()> {
        RsaPrivKeyMut::map_exponent(&self.0, closure)
    }

    fn take_exponent(&self) -> Option<&'static mut [u8]> {
        RsaPrivKeyMut::take_exponent(&self.0)
    }
}
