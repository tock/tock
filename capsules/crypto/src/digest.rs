//! Userspace message digests using mutex-protected digest peripherals.
//!
//! Read-only allow 0 contains the entire input; read-write allow 0 receives the
//! digest. Command 0 checks driver presence. Command 1 starts a digest, with
//! argument 0 selecting [`Algorithm`] and argument 1 unused. Subscribe 0 reports
//! `(status, digest_length, 0)`, with status 0 on success and an ErrorCode on
//! failure; digest_length is zero on failure. A synchronous command error does
//! not produce an upcall.
//!
//! One request may be outstanding across all processes, including time waiting
//! for a peripheral mutex. Other requests return BUSY. Unknown or unconfigured
//! algorithms return NOSUPPORT. A short output buffer returns SIZE. Empty input
//! is supported. Applications must keep both allows and their contents unchanged
//! until completion; input is read incrementally, not snapshotted. Buffer access
//! is checked again at every callback, and process exit releases the mutex when
//! the peripheral finishes.

use core::cell::Cell;

use capsules_core::driver_mutex::{
    DriverMutex, DriverMutexAny, DriverMutexClient, DriverMutexHandle, DriverMutexRef,
};
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::crypto::digest::{
    Digest, DigestAlgorithm, DigestClient, Md5, Sha1, Sha224, Sha256, Sha384, Sha512,
};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number for the multi-algorithm digest interface.
pub const DRIVER_NUM: usize = capsules_core::driver::NUM::Digest as usize;

/// Algorithm identifiers used by command 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Algorithm {
    Md5 = 0,
    Sha1 = 1,
    Sha224 = 2,
    Sha256 = 3,
    Sha384 = 4,
    Sha512 = 5,
}

impl TryFrom<usize> for Algorithm {
    type Error = ErrorCode;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Md5),
            1 => Ok(Self::Sha1),
            2 => Ok(Self::Sha224),
            3 => Ok(Self::Sha256),
            4 => Ok(Self::Sha384),
            5 => Ok(Self::Sha512),
            _ => Err(ErrorCode::NOSUPPORT),
        }
    }
}

impl Algorithm {
    /// Output length in bytes.
    pub fn digest_len(self) -> usize {
        match self {
            Self::Md5 => Md5::DIGEST_LEN,
            Self::Sha1 => Sha1::DIGEST_LEN,
            Self::Sha224 => Sha224::DIGEST_LEN,
            Self::Sha256 => Sha256::DIGEST_LEN,
            Self::Sha384 => Sha384::DIGEST_LEN,
            Self::Sha512 => Sha512::DIGEST_LEN,
        }
    }
}

/// Default type for an algorithm that has no peripheral configured.
pub enum UnavailableDigest {}

impl<AlgorithmType: DigestAlgorithm> Digest<AlgorithmType> for UnavailableDigest {
    fn digest(&self, _len: usize) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_client(&self, _client: &'static dyn DigestClient<AlgorithmType>) {}
}

/// One input allow, one output allow, and one completion upcall per process.
pub type DigestGrant = Grant<(), UpcallCount<1>, AllowRoCount<1>, AllowRwCount<1>>;

struct Slot<Driver: 'static> {
    mutex: Option<&'static DriverMutex<Driver>>,
    handle: OptionalCell<DriverMutexHandle>,
}

impl<Driver> Slot<Driver> {
    fn new(mutex: Option<&'static DriverMutex<Driver>>) -> Self {
        Self {
            mutex,
            handle: OptionalCell::empty(),
        }
    }

    fn register(&self, client: &'static dyn DriverMutexClient) -> Result<(), ErrorCode> {
        if let Some(mutex) = self.mutex {
            self.handle
                .set(mutex.add_client(client).ok_or(ErrorCode::NOMEM)?);
        }
        Ok(())
    }

    fn request(&self) -> Result<(), ErrorCode> {
        self.mutex
            .ok_or(ErrorCode::NOSUPPORT)?
            .request(self.handle.get().ok_or(ErrorCode::RESERVE)?)
    }
}

#[derive(Clone, Copy)]
struct Request {
    processid: ProcessId,
    algorithm: Algorithm,
    input_len: usize,
    offset: usize,
    written: bool,
}

enum ActiveDriver<
    Md5Driver: 'static,
    Sha1Driver: 'static,
    Sha224Driver: 'static,
    Sha256Driver: 'static,
    Sha384Driver: 'static,
    Sha512Driver: 'static,
> {
    Md5(DriverMutexRef<Md5Driver>),
    Sha1(DriverMutexRef<Sha1Driver>),
    Sha224(DriverMutexRef<Sha224Driver>),
    Sha256(DriverMutexRef<Sha256Driver>),
    Sha384(DriverMutexRef<Sha384Driver>),
    Sha512(DriverMutexRef<Sha512Driver>),
}

impl<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
    ActiveDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
{
    fn release(self) {
        match self {
            Self::Md5(guard) => drop(guard),
            Self::Sha1(guard) => drop(guard),
            Self::Sha224(guard) => drop(guard),
            Self::Sha256(guard) => drop(guard),
            Self::Sha384(guard) => drop(guard),
            Self::Sha512(guard) => drop(guard),
        }
    }
}

fn begin<AlgorithmType: DigestAlgorithm, Driver: Digest<AlgorithmType> + 'static>(
    resource: DriverMutexAny,
    client: &'static dyn DigestClient<AlgorithmType>,
    len: usize,
) -> Result<DriverMutexRef<Driver>, ErrorCode> {
    let driver = resource
        .downcast::<Driver>()
        .map_err(|_| ErrorCode::INVAL)?;
    driver.set_client(client);
    driver.digest(len)?;
    Ok(driver)
}

/// One digest syscall interface spanning independent or shared peripherals.
///
/// Multiple algorithm slots may refer to the same mutex. Boards must not
/// wrap a shared peripheral in different mutex instances. The active guard
/// is retained until completion, even if the requesting process exits.
pub struct DigestDriver<
    Md5Driver: Digest<Md5> + 'static = UnavailableDigest,
    Sha1Driver: Digest<Sha1> + 'static = UnavailableDigest,
    Sha224Driver: Digest<Sha224> + 'static = UnavailableDigest,
    Sha256Driver: Digest<Sha256> + 'static = UnavailableDigest,
    Sha384Driver: Digest<Sha384> + 'static = UnavailableDigest,
    Sha512Driver: Digest<Sha512> + 'static = UnavailableDigest,
> {
    apps: DigestGrant,
    md5: Slot<Md5Driver>,
    sha1: Slot<Sha1Driver>,
    sha224: Slot<Sha224Driver>,
    sha256: Slot<Sha256Driver>,
    sha384: Slot<Sha384Driver>,
    sha512: Slot<Sha512Driver>,
    request: Cell<Option<Request>>,
    active: MapCell<
        ActiveDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>,
    >,
}

impl<
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
> DigestDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
{
    /// Create the capsule with optional per-algorithm mutexes.
    ///
    /// Call [`Self::register`] after placing the capsule in static memory.
    pub fn new(
        apps: DigestGrant,
        md5: Option<&'static DriverMutex<Md5Driver>>,
        sha1: Option<&'static DriverMutex<Sha1Driver>>,
        sha224: Option<&'static DriverMutex<Sha224Driver>>,
        sha256: Option<&'static DriverMutex<Sha256Driver>>,
        sha384: Option<&'static DriverMutex<Sha384Driver>>,
        sha512: Option<&'static DriverMutex<Sha512Driver>>,
    ) -> Self {
        Self {
            apps,
            md5: Slot::new(md5),
            sha1: Slot::new(sha1),
            sha224: Slot::new(sha224),
            sha256: Slot::new(sha256),
            sha384: Slot::new(sha384),
            sha512: Slot::new(sha512),
            request: Cell::new(None),
            active: MapCell::empty(),
        }
    }

    /// Register once with every configured mutex, sharing a handle when
    /// several algorithms use the same mutex. Returns NOMEM if a mutex
    /// has no client capacity; earlier registrations remain installed.
    pub fn register(&'static self) -> Result<(), ErrorCode> {
        let client: &'static dyn DriverMutexClient = self;
        self.md5.register(client)?;
        self.sha1.register(client)?;
        self.sha224.register(client)?;
        self.sha256.register(client)?;
        self.sha384.register(client)?;
        self.sha512.register(client)?;
        Ok(())
    }

    fn start(&self, processid: ProcessId, algorithm: Algorithm) -> Result<(), ErrorCode> {
        let supported = match algorithm {
            Algorithm::Md5 => self.md5.mutex.is_some(),
            Algorithm::Sha1 => self.sha1.mutex.is_some(),
            Algorithm::Sha224 => self.sha224.mutex.is_some(),
            Algorithm::Sha256 => self.sha256.mutex.is_some(),
            Algorithm::Sha384 => self.sha384.mutex.is_some(),
            Algorithm::Sha512 => self.sha512.mutex.is_some(),
        };
        if !supported {
            return Err(ErrorCode::NOSUPPORT);
        }
        if self.request.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        let input_len = self
            .apps
            .enter(processid, |_, buffers| {
                let output = buffers
                    .get_readwrite_processbuffer(0)
                    .map_err(|_| ErrorCode::RESERVE)?;
                if output.len() < algorithm.digest_len() {
                    return Err(ErrorCode::SIZE);
                }
                let input = buffers
                    .get_readonly_processbuffer(0)
                    .map_err(|_| ErrorCode::RESERVE)?;
                Ok(input.len())
            })
            .map_err(ErrorCode::from)??;

        self.request.set(Some(Request {
            processid,
            algorithm,
            input_len,
            offset: 0,
            written: false,
        }));
        let result = match algorithm {
            Algorithm::Md5 => self.md5.request(),
            Algorithm::Sha1 => self.sha1.request(),
            Algorithm::Sha224 => self.sha224.request(),
            Algorithm::Sha256 => self.sha256.request(),
            Algorithm::Sha384 => self.sha384.request(),
            Algorithm::Sha512 => self.sha512.request(),
        };
        if result.is_err() {
            self.request.set(None);
        }
        result
    }

    fn read(&self, input: &mut [u8]) -> Result<usize, ErrorCode> {
        let mut request = self.request.get().ok_or(ErrorCode::OFF)?;
        let count = input.len().min(request.input_len - request.offset);
        if count == 0 {
            return Ok(0);
        }
        self.apps
            .enter(request.processid, |_, buffers| {
                buffers
                    .get_readonly_processbuffer(0)
                    .map_err(|_| ErrorCode::RESERVE)?
                    .enter(|source| {
                        if source.len() < request.input_len {
                            return Err(ErrorCode::SIZE);
                        }
                        source[request.offset..request.offset + count]
                            .copy_to_slice_or_err(&mut input[..count])
                    })
                    .map_err(|_| ErrorCode::RESERVE)?
            })
            .map_err(ErrorCode::from)??;
        request.offset += count;
        self.request.set(Some(request));
        Ok(count)
    }

    fn write(&self, digest: &[u8]) -> Result<(), ErrorCode> {
        let mut request = self.request.get().ok_or(ErrorCode::OFF)?;
        if request.written || request.offset != request.input_len {
            return Err(ErrorCode::FAIL);
        }
        if digest.len() != request.algorithm.digest_len() {
            return Err(ErrorCode::SIZE);
        }
        self.apps
            .enter(request.processid, |_, buffers| {
                buffers
                    .get_readwrite_processbuffer(0)
                    .map_err(|_| ErrorCode::RESERVE)?
                    .mut_enter(|output| {
                        if output.len() < digest.len() {
                            return Err(ErrorCode::SIZE);
                        }
                        output[..digest.len()].copy_from_slice_or_err(digest)
                    })
                    .map_err(|_| ErrorCode::RESERVE)?
            })
            .map_err(ErrorCode::from)??;
        request.written = true;
        self.request.set(Some(request));
        Ok(())
    }

    fn finish(&self, result: Result<(), ErrorCode>) {
        let request = self.request.take();
        self.active.take().map(ActiveDriver::release);
        if let Some(request) = request {
            let result = result.and(if request.written {
                Ok(())
            } else {
                Err(ErrorCode::FAIL)
            });
            let length = if result.is_ok() {
                request.algorithm.digest_len()
            } else {
                0
            };
            let _ = self.apps.enter(request.processid, |_, buffers| {
                let _ = buffers.schedule_upcall(0, (into_statuscode(result), length, 0));
            });
        }
    }
}

impl<
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
> SyscallDriver
    for DigestDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
{
    fn command(
        &self,
        command_num: usize,
        algorithm: usize,
        _unused: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => Algorithm::try_from(algorithm)
                .and_then(|algorithm| self.start(processid, algorithm))
                .into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
> DriverMutexClient
    for DigestDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
{
    fn ready(&'static self, resource: DriverMutexAny) {
        let Some(request) = self.request.get() else {
            return;
        };
        let result = self
            .apps
            .enter(request.processid, |_, _| ())
            .map_err(ErrorCode::from);
        if let Err(error) = result {
            drop(resource);
            self.finish(Err(error));
            return;
        }
        let result = match request.algorithm {
            Algorithm::Md5 => {
                begin::<Md5, Md5Driver>(resource, self, request.input_len).map(ActiveDriver::Md5)
            }
            Algorithm::Sha1 => {
                begin::<Sha1, Sha1Driver>(resource, self, request.input_len).map(ActiveDriver::Sha1)
            }
            Algorithm::Sha224 => begin::<Sha224, Sha224Driver>(resource, self, request.input_len)
                .map(ActiveDriver::Sha224),
            Algorithm::Sha256 => begin::<Sha256, Sha256Driver>(resource, self, request.input_len)
                .map(ActiveDriver::Sha256),
            Algorithm::Sha384 => begin::<Sha384, Sha384Driver>(resource, self, request.input_len)
                .map(ActiveDriver::Sha384),
            Algorithm::Sha512 => begin::<Sha512, Sha512Driver>(resource, self, request.input_len)
                .map(ActiveDriver::Sha512),
        };
        match result {
            Ok(driver) => {
                self.active.replace(driver);
            }
            Err(error) => self.finish(Err(error)),
        }
    }
}

impl<
    AlgorithmType: DigestAlgorithm,
    Md5Driver: Digest<Md5> + 'static,
    Sha1Driver: Digest<Sha1> + 'static,
    Sha224Driver: Digest<Sha224> + 'static,
    Sha256Driver: Digest<Sha256> + 'static,
    Sha384Driver: Digest<Sha384> + 'static,
    Sha512Driver: Digest<Sha512> + 'static,
> DigestClient<AlgorithmType>
    for DigestDriver<Md5Driver, Sha1Driver, Sha224Driver, Sha256Driver, Sha384Driver, Sha512Driver>
{
    fn read_input(&self, input: &mut [u8]) -> Result<usize, ErrorCode> {
        self.read(input)
    }

    fn write_digest(&self, digest: &AlgorithmType::Output) -> Result<(), ErrorCode> {
        self.write(digest.as_ref())
    }

    fn digest_done(&self, result: Result<(), ErrorCode>) {
        self.finish(result);
    }
}
