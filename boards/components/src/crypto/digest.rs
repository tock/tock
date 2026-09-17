//! Builder for the multi-algorithm digest syscall capsule.
//!
//! Start with [`DigestComponent::new`], then supply only the available algorithms.
//! When one peripheral implements several algorithms, pass the same DriverMutex
//! to their builder methods. Each distinct mutex needs one client slot for this
//! capsule and must already have its deferred call registered (for example by
//! DriverMutexComponent). Do not construct multiple mutexes for one peripheral.
//!
//! The driver number is board-assigned. This API is not compatible with the
//! legacy SHA syscall driver and must not silently replace it under its number.
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
use kernel::hil::crypto::digest::{Digest, Md5, Sha1, Sha224, Sha256, Sha384, Sha512};

/// Allocate a digest capsule, naming only configured driver types in algorithm
/// order (md5, sha1, sha224, sha256, sha384, sha512), each followed by a comma.
#[macro_export]
macro_rules! digest_component_static {
    (
        $(md5: $md5:ty,)?
        $(sha1: $sha1:ty,)?
        $(sha224: $sha224:ty,)?
        $(sha256: $sha256:ty,)?
        $(sha384: $sha384:ty,)?
        $(sha512: $sha512:ty,)?
    ) => {
        kernel::static_buf!($crate::crypto::digest::DigestDriver<
            $crate::digest_component_static!(@driver $($md5)?),
            $crate::digest_component_static!(@driver $($sha1)?),
            $crate::digest_component_static!(@driver $($sha224)?),
            $crate::digest_component_static!(@driver $($sha256)?),
            $crate::digest_component_static!(@driver $($sha384)?),
            $crate::digest_component_static!(@driver $($sha512)?),
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
    Sha512Driver: Digest<Sha512> + 'static = UnavailableDigest,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: Cap,
    md5: Option<&'static DriverMutex<Md5Driver>>,
    sha1: Option<&'static DriverMutex<Sha1Driver>>,
    sha224: Option<&'static DriverMutex<Sha224Driver>>,
    sha256: Option<&'static DriverMutex<Sha256Driver>>,
    sha384: Option<&'static DriverMutex<Sha384Driver>>,
    sha512: Option<&'static DriverMutex<Sha512Driver>>,
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
    Sha512Driver: Digest<Sha512> + 'static,
>
    DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512Driver,
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
        Sha512Driver,
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
        }
    }

    /// Configure the SHA-1 mutex, retaining all other algorithm slots.
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
        Sha512Driver,
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
        }
    }

    /// Configure the SHA-224 mutex, retaining all other algorithm slots.
    pub fn with_sha224<Driver: Digest<Sha224> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<Cap, Md5Driver, Sha1Driver, Driver, Sha256Driver, Sha384Driver, Sha512Driver>
    {
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
        }
    }

    /// Configure the SHA-256 mutex, retaining all other algorithm slots.
    pub fn with_sha256<Driver: Digest<Sha256> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<Cap, Md5Driver, Sha1Driver, Sha224Driver, Driver, Sha384Driver, Sha512Driver>
    {
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
        }
    }

    /// Configure the SHA-384 mutex, retaining all other algorithm slots.
    pub fn with_sha384<Driver: Digest<Sha384> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<Cap, Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Driver, Sha512Driver>
    {
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
        }
    }

    /// Configure the SHA-512 mutex, retaining all other algorithm slots.
    pub fn with_sha512<Driver: Digest<Sha512> + 'static>(
        self,
        mutex: &'static DriverMutex<Driver>,
    ) -> DigestComponent<Cap, Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Driver>
    {
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
    Sha512Driver: Digest<Sha512> + 'static,
> Component
    for DigestComponent<
        Cap,
        Md5Driver,
        Sha1Driver,
        Sha224Driver,
        Sha256Driver,
        Sha384Driver,
        Sha512Driver,
    >
{
    type StaticInput = &'static mut MaybeUninit<
        DigestDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>,
    >;
    type Output = Result<
        &'static DigestDriver<
            Md5Driver,
            Sha1Driver,
            Sha224Driver,
            Sha256Driver,
            Sha384Driver,
            Sha512Driver,
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
            self.sha512,
        ));
        driver.register()?;
        Ok(driver)
    }
}
