//! Builder for the multi-algorithm digest syscall capsule.
//!
//! Start with [`DigestComponent::new`], then supply only the available algorithms.
//! When one peripheral implements several algorithms, pass the same DriverMutex
//! to their builder methods. Each distinct mutex needs one client slot for this
//! capsule and must already have its deferred call registered (for example by
//! DriverMutexComponent). Do not construct multiple mutexes for one peripheral.
//!
//! Use [`capsules_crypto::digest::DRIVER_NUM`] for the driver number. This API is
//! not compatible with the legacy SHA or HMAC syscall drivers.
//!
//! ```rust,ignore
//! let digest = components::crypto::digest::DigestComponent::new(board_kernel, driver_num, mem_cap)
//!     .with_sha224(hash_mutex)
//!     .with_sha256(hash_mutex)
//!     .finalize(components::digest_component_static!(
//!         sha224: ChipHash,
//!         sha256: ChipHash,
//!     ))
//!     .expect("digest mutex client capacity");
//! ```

use capsules_core::driver_mutex::DriverMutex;
pub use capsules_crypto::digest::{DigestDriver, UnavailableDigest};
use core::mem::MaybeUninit;
use kernel::ErrorCode;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil::crypto::digest::{
    Digest, Md5, Sha1, Sha3_224, Sha3_256, Sha3_384, Sha3_512, Sha224, Sha256, Sha384, Sha512,
    Sha512_224, Sha512_256, Sm3,
};

/// Allocate a digest capsule, naming only configured driver types in algorithm
/// order (md5, sha1, sha224, sha256, sha384, sha512_224, sha512_256, sha512,
/// sha3_224, sha3_256, sha3_384, sha3_512, sm3), each followed by a comma.
#[macro_export]
macro_rules! digest_component_static {
    (
        $(md5: $md5:ty,)?
        $(sha1: $sha1:ty,)?
        $(sha224: $sha224:ty,)?
        $(sha256: $sha256:ty,)?
        $(sha384: $sha384:ty,)?
        $(sha512_224: $sha512_224:ty,)?
        $(sha512_256: $sha512_256:ty,)?
        $(sha512: $sha512:ty,)?
        $(sha3_224: $sha3_224:ty,)?
        $(sha3_256: $sha3_256:ty,)?
        $(sha3_384: $sha3_384:ty,)?
        $(sha3_512: $sha3_512:ty,)?
        $(sm3: $sm3:ty,)?
    ) => {
        kernel::static_buf!($crate::crypto::digest::DigestDriver<
            $crate::digest_component_static!(@driver $($md5)?),
            $crate::digest_component_static!(@driver $($sha1)?),
            $crate::digest_component_static!(@driver $($sha224)?),
            $crate::digest_component_static!(@driver $($sha256)?),
            $crate::digest_component_static!(@driver $($sha384)?),
            $crate::digest_component_static!(@driver $($sha512_224)?),
            $crate::digest_component_static!(@driver $($sha512_256)?),
            $crate::digest_component_static!(@driver $($sha512)?),
            $crate::digest_component_static!(@driver $($sha3_224)?),
            $crate::digest_component_static!(@driver $($sha3_256)?),
            $crate::digest_component_static!(@driver $($sha3_384)?),
            $crate::digest_component_static!(@driver $($sha3_512)?),
            $crate::digest_component_static!(@driver $($sm3)?),
        >)
    };
    (@driver $driver:ty) => { $driver };
    (@driver) => { $crate::crypto::digest::UnavailableDigest };
}

/// A type-changing builder whose unconfigured algorithms default to unavailable.
pub struct DigestComponent<
    Cap: MemoryAllocationCapability,
    Md5Driver: Digest<Md5> + 'static = UnavailableDigest,
    Sha1Driver: Digest<Sha1> + 'static = UnavailableDigest,
    Sha224Driver: Digest<Sha224> + 'static = UnavailableDigest,
    Sha256Driver: Digest<Sha256> + 'static = UnavailableDigest,
    Sha384Driver: Digest<Sha384> + 'static = UnavailableDigest,
    Sha512_224Driver: Digest<Sha512_224> + 'static = UnavailableDigest,
    Sha512_256Driver: Digest<Sha512_256> + 'static = UnavailableDigest,
    Sha512Driver: Digest<Sha512> + 'static = UnavailableDigest,
    Sha3_224Driver: Digest<Sha3_224> + 'static = UnavailableDigest,
    Sha3_256Driver: Digest<Sha3_256> + 'static = UnavailableDigest,
    Sha3_384Driver: Digest<Sha3_384> + 'static = UnavailableDigest,
    Sha3_512Driver: Digest<Sha3_512> + 'static = UnavailableDigest,
    Sm3Driver: Digest<Sm3> + 'static = UnavailableDigest,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: Cap,
    md5: Option<&'static DriverMutex<Md5Driver>>,
    sha1: Option<&'static DriverMutex<Sha1Driver>>,
    sha224: Option<&'static DriverMutex<Sha224Driver>>,
    sha256: Option<&'static DriverMutex<Sha256Driver>>,
    sha384: Option<&'static DriverMutex<Sha384Driver>>,
    sha512_224: Option<&'static DriverMutex<Sha512_224Driver>>,
    sha512_256: Option<&'static DriverMutex<Sha512_256Driver>>,
    sha512: Option<&'static DriverMutex<Sha512Driver>>,
    sha3_224: Option<&'static DriverMutex<Sha3_224Driver>>,
    sha3_256: Option<&'static DriverMutex<Sha3_256Driver>>,
    sha3_384: Option<&'static DriverMutex<Sha3_384Driver>>,
    sha3_512: Option<&'static DriverMutex<Sha3_512Driver>>,
    sm3: Option<&'static DriverMutex<Sm3Driver>>,
}

impl<Cap: MemoryAllocationCapability> DigestComponent<Cap> {
    /// Start a builder with no supported algorithms.
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, mem_cap: Cap) -> Self {
        Self {
            board_kernel,
            driver_num,
            mem_cap,
            md5: None,
            sha1: None,
            sha224: None,
            sha256: None,
            sha384: None,
            sha512: None,
            sha512_224: None,
            sha512_256: None,
            sha3_224: None,
            sha3_256: None,
            sha3_384: None,
            sha3_512: None,
            sm3: None,
        }
    }
}

impl<
    Cap: MemoryAllocationCapability,
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512_224Driver: Digest<Sha512_224> + 'static,
    Sha512_256Driver: Digest<Sha512_256> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
    Sha3_224Driver: Digest<Sha3_224> + 'static,
    Sha3_256Driver: Digest<Sha3_256> + 'static,
    Sha3_384Driver: Digest<Sha3_384> + 'static,
    Sha3_512Driver: Digest<Sha3_512> + 'static,
    Sm3Driver: Digest<Sm3> + 'static,
>
    DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    >
{
    /// Configure the MD5 mutex, retaining all other algorithm slots.
    pub fn with_md5<Driver: Digest<Md5> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: Some(mutex),
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA1 mutex, retaining all other algorithm slots.
    pub fn with_sha1<Driver: Digest<Sha1> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: Some(mutex),
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA224 mutex, retaining all other algorithm slots.
    pub fn with_sha224<Driver: Digest<Sha224> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: Some(mutex),
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA256 mutex, retaining all other algorithm slots.
    pub fn with_sha256<Driver: Digest<Sha256> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: Some(mutex),
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA384 mutex, retaining all other algorithm slots.
    pub fn with_sha384<Driver: Digest<Sha384> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: Some(mutex),
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA512/224 mutex, retaining all other algorithm slots.
    pub fn with_sha512_224<Driver: Digest<Sha512_224> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: Some(mutex),
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA512/256 mutex, retaining all other algorithm slots.
    pub fn with_sha512_256<Driver: Digest<Sha512_256> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: Some(mutex),
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA512 mutex, retaining all other algorithm slots.
    pub fn with_sha512<Driver: Digest<Sha512> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: Some(mutex),
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA3-224 mutex, retaining all other algorithm slots.
    pub fn with_sha3_224<Driver: Digest<Sha3_224> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: Some(mutex),
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA3-256 mutex, retaining all other algorithm slots.
    pub fn with_sha3_256<Driver: Digest<Sha3_256> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: Some(mutex),
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA3-384 mutex, retaining all other algorithm slots.
    pub fn with_sha3_384<Driver: Digest<Sha3_384> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Driver,
        Sha3_512Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: Some(mutex),
            sha3_512: self.sha3_512,
            sm3: self.sm3,
        }
    }

    /// Configure the SHA3-512 mutex, retaining all other algorithm slots.
    pub fn with_sha3_512<Driver: Digest<Sha3_512> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Driver,
        Sm3Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: Some(mutex),
            sm3: self.sm3,
        }
    }

    /// Configure the SM3 mutex, retaining all other algorithm slots.
    pub fn with_sm3<Driver: Digest<Sm3> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Driver,
    > {
        DigestComponent {
            board_kernel: self.board_kernel,
            driver_num: self.driver_num,
            mem_cap: self.mem_cap,
            md5: self.md5,
            sha1: self.sha1,
            sha224: self.sha224,
            sha256: self.sha256,
            sha384: self.sha384,
            sha512: self.sha512,
            sha512_224: self.sha512_224,
            sha512_256: self.sha512_256,
            sha3_224: self.sha3_224,
            sha3_256: self.sha3_256,
            sha3_384: self.sha3_384,
            sha3_512: self.sha3_512,
            sm3: Some(mutex),
        }
    }
}

impl<
    Cap: MemoryAllocationCapability,
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512_224Driver: Digest<Sha512_224> + 'static,
    Sha512_256Driver: Digest<Sha512_256> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
    Sha3_224Driver: Digest<Sha3_224> + 'static,
    Sha3_256Driver: Digest<Sha3_256> + 'static,
    Sha3_384Driver: Digest<Sha3_384> + 'static,
    Sha3_512Driver: Digest<Sha3_512> + 'static,
    Sm3Driver: Digest<Sm3> + 'static,
> Component
    for DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512_224Driver,
        Sha512_256Driver,
        Sha512Driver,
        Sha3_224Driver,
        Sha3_256Driver,
        Sha3_384Driver,
        Sha3_512Driver,
        Sm3Driver,
    >
{
    type StaticInput = &'static mut MaybeUninit<
        DigestDriver<
            Md5Driver,
            Sha1Driver,
            Sha224Driver,
            Sha256Driver,
            Sha384Driver,
            Sha512_224Driver,
            Sha512_256Driver,
            Sha512Driver,
            Sha3_224Driver,
            Sha3_256Driver,
            Sha3_384Driver,
            Sha3_512Driver,
            Sm3Driver,
        >,
    >;
    type Output = Result<
        &'static DigestDriver<
            Md5Driver,
            Sha1Driver,
            Sha224Driver,
            Sha256Driver,
            Sha384Driver,
            Sha512_224Driver,
            Sha512_256Driver,
            Sha512Driver,
            Sha3_224Driver,
            Sha3_256Driver,
            Sha3_384Driver,
            Sha3_512Driver,
            Sm3Driver,
        >,
        ErrorCode,
    >;

    /// Initialize and register the capsule. NOMEM indicates insufficient mutex
    /// client capacity; any registrations already made remain installed.
    fn finalize(self, storage: Self::StaticInput) -> Self::Output {
        let driver = storage.write(DigestDriver::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.md5,
            self.sha1,
            self.sha224,
            self.sha256,
            self.sha384,
            self.sha512_224,
            self.sha512_256,
            self.sha512,
            self.sha3_224,
            self.sha3_256,
            self.sha3_384,
            self.sha3_512,
            self.sm3,
        ));
        driver.register()?;
        Ok(driver)
    }
}
