// (todo) add copyright header
use capsules_core::{aes::Aes128Ctr, driver};
pub const DRIVER_NUM: usize = driver::NUM::Aes as usize;
use capsules_core::aes::Aes128Ecb;
use kernel::{
    grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount},
    hil::symmetric_encryption,
    processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer},
    syscall::{CommandReturn, SyscallDriver},
    utilities::{
        cells::{OptionalCell, TakeCell},
        leasable_buffer::SubSliceMut,
    },
    ErrorCode, ProcessId,
};

// each app can only do 1 aes op at a time
mod ro_allow {
    pub const IV: usize = 0;
    pub const KEY: usize = 1;
    pub const SRC_BUF: usize = 2;
    pub const COUNT: u8 = 3;
}

mod rw_allow {
    pub const DST_BUF: usize = 0;
    pub const COUNT: u8 = 2;
}

mod upcall {
    pub const CRYPT_DONE: usize = 0;
    pub const COUNT: u8 = 1;
}

pub struct AesOperators<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> {
    ecb: &'a ECB,
    ctr: &'a CTR,
}

impl<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> AesOperators<'a, ECB, CTR> {
    pub fn new(ecb: &'a ECB, ctr: &'a CTR) -> Self {
        AesOperators { ecb, ctr }
    }
}

// (todo) this is currently a wip prototype to demonstrate how
// the updated interface greatly simplifies writing the syscall
// driver. The driver will require `A` to implement CTR CCM GCM etc as well.
// Only require Ecb / Ctr for now for simplicity and to avoid
// having to update all other interfaces. We select Ecb / Ctr
// since the prototype is for nrf5x hardware (which supports hw
// ecb) and we build ctr in software to demonstrate updated
// crypto interface in hw support and sw support for crypto
// modes.
pub struct AesDriver<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> {
    aes: AesOperators<'a, ECB, CTR>,
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,
    kernel_dest_buf: TakeCell<'static, [u8]>,
    kernel_src_buf: TakeCell<'static, [u8]>,
}

impl<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> AesDriver<'a, ECB, CTR> {
    pub fn new(
        aes_ecb: &'a ECB,
        aes_ctr: &'a CTR,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        kernel_dest_buf: &'static mut [u8],
        kernel_src_buf: &'static mut [u8],
    ) -> Self {
        AesDriver {
            aes: AesOperators::new(aes_ecb, aes_ctr),
            apps: grant,
            current_process: OptionalCell::empty(),
            kernel_dest_buf: TakeCell::new(kernel_dest_buf),
            kernel_src_buf: TakeCell::new(kernel_src_buf),
        }
    }

    fn enqueue(&self, pid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(pid, |app, _| {
            // Determine if this app is already queued for work and that the
            // AES mode has been set.
            if !app.pending || app.aes_mode.is_some() {
                app.pending = true;
                Ok(())
            } else {
                Err(ErrorCode::BUSY)
            }
        })??;

        if self.current_process.is_none() {
            for cntr in self.apps.iter() {
                let processid = cntr.processid();
                let mut res = None;
                cntr.enter(|mut app, kernel_data| {
                    if app.pending {
                        self.current_process.set(processid);
                        res = self.app_crypt_op(&mut app, kernel_data);
                    }
                });

                if res.is_some() {
                    break;
                }
            }
        }
        Ok(())
    }

    fn perform_crypt(
        &self,
        mode: AesMode,
        iv: Option<&[u8; symmetric_encryption::AES128_BLOCK_SIZE]>,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        src: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        match mode {
            AesMode::AesEcb => {
                if let Err(error_code) = self.aes.ecb.setup_cipher(key) {
                    return Err((error_code, src, dest));
                }

                self.aes.ecb.crypt(src, dest)
            }
            AesMode::AesCtr => {
                let iv = if let Some(iv) = iv {
                    iv
                } else {
                    // CTR mode requires an IV, so return error if not provided.
                    return Err((ErrorCode::INVAL, src, dest));
                };

                if let Err(error_code) = self.aes.ctr.setup_cipher(key, iv) {
                    return Err((error_code, src, dest));
                }

                self.aes.ctr.crypt(src, dest)
            }
            AesMode::AesCbc => {
                todo!()
            }
            AesMode::AesCcm => {
                todo!()
            }
            AesMode::AesGcm => {
                todo!()
            }
        }
    }

    fn app_crypt_op(&self, app: &mut App, kernel_data: &GrantKernelData) -> Option<()> {
        app.pending = false;
        let aes_mode = if let Some(mode) = app.aes_mode {
            mode
        } else {
            // AES mode not set, so no encryption operation can be performed.
            // Short-circuit here and mark this app as no longer having work.
            let _ =
                kernel_data.schedule_upcall(upcall::CRYPT_DONE, (ErrorCode::INVAL.into(), 0, 0));
            return None;
        };

        // Obtain key shared from userspace.
        let key_copy_result = kernel_data
            .get_readonly_processbuffer(ro_allow::KEY)
            .and_then(|key_processbuffer| {
                key_processbuffer.enter(|user_key| {
                    let mut key = [0u8; symmetric_encryption::AES128_KEY_SIZE];
                    if let Err(_) = user_key.copy_to_slice_or_err(&mut key) {
                        return Err(kernel::process::Error::KernelError);
                    };

                    Ok(key)
                })
            });

        let key = if let Ok(Ok(key)) = key_copy_result {
            key
        } else {
            // Without a key, no encryption mode will succeed, so short-circuit
            // here and mark this app as no longer having work.
            let _ =
                kernel_data.schedule_upcall(upcall::CRYPT_DONE, (ErrorCode::INVAL.into(), 0, 0));
            return None;
        };

        // Obtain IV shared from userspace (not needed for ECB so not necessarily an error
        // if not shared).
        let iv = kernel_data
            .get_readonly_processbuffer(ro_allow::IV)
            .map_or_else(
                |_| None,
                |iv_processbuf| {
                    iv_processbuf
                        .enter(|user_iv| {
                            let mut iv = [0u8; symmetric_encryption::AES128_BLOCK_SIZE];
                            if let Err(_) = user_iv.copy_to_slice_or_err(&mut iv) {
                                return Err(kernel::process::Error::KernelError);
                            }

                            Ok(iv)
                        })
                        .map_or_else(
                            // Upon error, treat IV as not provided. If the requested
                            // AES mode requires an IV, the error will be caught later.
                            |_| None,
                            |iv_res| match iv_res {
                                Ok(iv) => Some(iv),
                                Err(_) => None,
                            },
                        )
                },
            );

        let kernel_src_buf = self.kernel_src_buf.take().unwrap();
        let kernel_dest_buf = self.kernel_dest_buf.take().unwrap();

        // Obtain dest (and possibly src) buffers shared from userspace.
        let buffer_copy_result = kernel_data
            .get_readwrite_processbuffer(rw_allow::DST_BUF)
            .and_then(|dst_processbuf| {
                dst_processbuf
                    .enter(|user_dst_buf| {
                        // Dest buffer must be provided and fit in the kernel buffer.
                        if user_dst_buf.len() == 0 {
                            return Err(kernel::process::Error::KernelError);
                        }

                        user_dst_buf
                            .copy_to_slice_or_err(&mut kernel_dest_buf[..user_dst_buf.len()])
                            .map_err(|_| kernel::process::Error::KernelError)
                    })
                    .and_then(|_| {
                        // Now see if user has provided a src buf.
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::SRC_BUF)
                            .and_then(|src_processbuf| {
                                src_processbuf.enter(|user_src_buf| {
                                    if user_src_buf.len() == 0 {
                                        // no src buffer provided, this is not an error,
                                        // as it is valid to not provide a src buffer.
                                        self.kernel_src_buf.replace(kernel_src_buf);
                                        return Ok(None);
                                    }

                                    // If a src buffer is provided, but is too large for the
                                    // kernel buffer, return an error.
                                    if let Err(_) = user_src_buf.copy_to_slice_or_err(
                                        &mut kernel_src_buf[..user_src_buf.len()],
                                    ) {
                                        self.kernel_src_buf.replace(kernel_src_buf);
                                        Err(kernel::process::Error::OutOfMemory)
                                    } else {
                                        Ok(Some(kernel_src_buf))
                                    }
                                })
                            })
                    })
            });

        match buffer_copy_result {
            Ok(Ok(src_buf)) => {
                let src = src_buf.map(|buf| SubSliceMut::new(buf));
                let dest = SubSliceMut::new(kernel_dest_buf);
                self.perform_crypt(aes_mode, iv.as_ref(), &key, src, dest)
                    .map_or_else(
                        |(code, src, dest)| {
                            self.kernel_dest_buf.replace(dest.take());
                            if let Some(src) = src {
                                self.kernel_src_buf.replace(src.take());
                            }

                            // an error occurred during crypt
                            let _ = kernel_data
                                .schedule_upcall(upcall::CRYPT_DONE, (code.into(), 0, 0));
                            None
                        },
                        |_| {
                            // Success, AES operation is in progress.
                            Some(())
                        },
                    )
            }
            _ => {
                // Unable to obtain dest buffer, so no encryption operation can be
                // performed.  Short-circuit here and mark this app as no longer
                // having work.
                let _ = kernel_data
                    .schedule_upcall(upcall::CRYPT_DONE, (ErrorCode::INVAL.into(), 0, 0));
                None
            }
        }
    }
}

#[derive(Copy, Clone)]
enum AesMode {
    AesEcb,
    AesCtr,
    AesCbc,
    AesCcm,
    AesGcm,
}

impl TryFrom<usize> for AesMode {
    type Error = ErrorCode;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AesMode::AesEcb),
            1 => Ok(AesMode::AesCtr),
            2 => Ok(AesMode::AesCbc),
            3 => Ok(AesMode::AesCcm),
            4 => Ok(AesMode::AesGcm),
            _ => Err(ErrorCode::INVAL),
        }
    }
}

pub struct App {
    aes_mode: Option<AesMode>,
    pending: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            aes_mode: None,
            pending: false,
        }
    }
}

impl<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> symmetric_encryption::Client<'a>
    for AesDriver<'a, ECB, CTR>
{
    fn crypt_done(
        &'a self,
        _source: Option<SubSliceMut<'static, u8>>,
        dest: Result<SubSliceMut<'static, u8>, (ErrorCode, SubSliceMut<'static, u8>)>,
    ) {
        let pid = self
            .current_process
            .take()
            .expect("[AES Driver] invalid state.");

        // (todo) add error handling
        let _ = self.apps.enter(pid, |_, kernel_data| {
            // Determine if an error occured in crypt.
            let res = match dest {
                Ok(mut dest) => {
                    // Copy dest buffer to userspace rw allow buffer.
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::DST_BUF)
                        .and_then(|rw_processbuf| {
                            rw_processbuf.mut_enter(|buf| {
                                dest.slice(..buf.len());
                                buf.copy_from_slice_or_err(dest.as_slice())
                            })
                        })
                        .map_or_else(|error| Err(error.into()), |res| res)
                }
                Err((error_code, _)) => Err(error_code),
            };

            // Notify app of crypt operation outcome.
            // (TODO) fix / add correct values to pass since 0,0 is not correct.
            if let Err(error_code) = res {
                let _ = kernel_data.schedule_upcall(upcall::CRYPT_DONE, (error_code.into(), 0, 0));
            } else {
                // todo check values for r2/r3
                let _ = kernel_data.schedule_upcall(
                    upcall::CRYPT_DONE,
                    (kernel::errorcode::into_statuscode(Ok(())), 0, 0),
                );
            }
        });
    }
}

impl<'a, ECB: Aes128Ecb<'a>, CTR: Aes128Ctr<'a>> SyscallDriver for AesDriver<'a, ECB, CTR> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Existence Check
            0 => CommandReturn::success(),

            // Perform crypt operation with the provided mode.
            1 => {
                // Get aes mode from provided data1 arg
                let mode = data1.try_into();
                if let Ok(mode) = mode {
                    let _ = self.apps.enter(processid, |app, _| {
                        app.aes_mode = Some(mode);
                    });
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }
            2 => match self.enqueue(processid) {
                Ok(()) => CommandReturn::success(),
                Err(e) => CommandReturn::failure(e),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}
