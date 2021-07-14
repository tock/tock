//! Provides userspace access to a Crc unit.
//!
//! ## Instantiation
//!
//! Instantiate the capsule for use as a system call driver with a hardware
//! implementation and a `Grant` for the `App` type, and set the result as a
//! client of the hardware implementation. For example, using the SAM4L's `CrcU`
//! driver:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let crc_buffer = static_init!([u8; 64], [0; 64]);
//!
//! let crc = static_init!(
//!     capsules::crc::CrcDriver<'static, sam4l::crccu::Crccu<'static>>,
//!     capsules::crc::CrcDriver::new(
//!         &mut sam4l::crccu::CRCCU,
//!         crc_buffer,
//!         board_kernel.create_grant(&grant_cap)
//!      )
//! );
//! sam4l::crccu::CRCCU.set_client(crc);
//!
//! ```
//!
//! ## Crc Algorithms
//!
//! The capsule supports two general purpose Crc algorithms, as well as a few
//! hardware specific algorithms implemented on the Atmel SAM4L.
//!
//! In the values used to identify polynomials below, more-significant bits
//! correspond to higher-order terms, and the most significant bit is omitted
//! because it always equals one.  All algorithms listed here consume each input
//! byte from most-significant bit to least-significant.
//!
//! ### Crc-32
//!
//! __Polynomial__: `0x04C11DB7`
//!
//! This algorithm is used in Ethernet and many other applications. It bit-
//! reverses and then bit-inverts the output.
//!
//! ### Crc-32C
//!
//! __Polynomial__: `0x1EDC6F41`
//!
//! Bit-reverses and then bit-inverts the output. It *may* be equivalent to
//! various Crc functions using the same name.
//!
//! ### SAM4L-16
//!
//! __Polynomial__: `0x1021`
//!
//! This algorithm does no post-processing on the output value. The sixteen-bit
//! Crc result is placed in the low-order bits of the returned result value, and
//! the high-order bits will all be set.  That is, result values will always be
//! of the form `0xFFFFxxxx` for this algorithm.  It can be performed purely in
//! hardware on the SAM4L.
//!
//! ### SAM4L-32
//!
//! __Polynomial__: `0x04C11DB7`
//!
//! This algorithm uses the same polynomial as `Crc-32`, but does no post-
//! processing on the output value.  It can be perfomed purely in hardware on
//! the SAM4L.
//!
//! ### SAM4L-32C
//!
//! __Polynomial__: `0x1EDC6F41`
//!
//! This algorithm uses the same polynomial as `Crc-32C`, but does no post-
//! processing on the output value.  It can be performed purely in hardware on
//! the SAM4L.

use core::cell::Cell;
use core::{cmp, mem};

use kernel::grant::Grant;
use kernel::hil::crc::{Client, Crc, CrcAlgorithm, CrcOutput};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer, ReadableProcessSlice};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::NumericCellExt;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Crc as usize;
pub const DEFAULT_CRC_BUF_LENGTH: usize = 256;

/// An opaque value maintaining state for one application's request
#[derive(Default)]
pub struct App {
    buffer: ReadOnlyProcessBuffer,
    // if Some, the process is waiting for the result of CRC
    // of len bytes using the given algorithm
    request: Option<(CrcAlgorithm, usize)>,
}

/// Struct that holds the state of the Crc driver and implements the `Driver` trait for use by
/// processes through the system call interface.
pub struct CrcDriver<'a, C: Crc<'a>> {
    crc: &'a C,
    crc_buffer: TakeCell<'static, [u8]>,
    grant: Grant<App, 1>,
    current_process: OptionalCell<ProcessId>,
    // We need to save our current
    app_buffer_written: Cell<usize>,
}

impl<'a, C: Crc<'a>> CrcDriver<'a, C> {
    /// Create a `Crc` driver
    ///
    /// The argument `crc_unit` must implement the abstract `Crc`
    /// hardware interface.  The argument `apps` should be an empty
    /// kernel `Grant`, and will be used to track application
    /// requests.
    ///
    /// ## Example
    ///
    /// ```rust
    /// capsules::crc::Crc::new(&sam4l::crccu::CrcCU, board_kernel.create_grant(&grant_cap));
    /// ```
    ///
    pub fn new(
        crc: &'a C,
        crc_buffer: &'static mut [u8],
        grant: Grant<App, 1>,
    ) -> CrcDriver<'a, C> {
        CrcDriver {
            crc,
            crc_buffer: TakeCell::new(crc_buffer),
            grant,
            current_process: OptionalCell::empty(),
            app_buffer_written: Cell::new(0),
        }
    }

    fn do_next_input(&self, data: &ReadableProcessSlice, len: usize) -> usize {
        let count = self.crc_buffer.take().map_or(0, |kbuffer| {
            let copy_len = cmp::min(len, kbuffer.len());
            for i in 0..copy_len {
                kbuffer[i] = data[i].get();
            }
            if copy_len > 0 {
                let mut leasable = LeasableBuffer::new(kbuffer);
                leasable.slice(0..copy_len);
                let res = self.crc.input(leasable);
                match res {
                    Ok(()) => copy_len,
                    Err((_err, leasable)) => {
                        self.crc_buffer.put(Some(leasable.take()));
                        0
                    }
                }
            } else {
                0
            }
        });
        count
    }

    // Start a new request. Return Ok(()) if one started, Err(FAIL) if not.
    // Issue callbacks for any requests that are invalid, either because
    // they are zero-length or requested an invalid algoritm.
    fn next_request(&self) -> Result<(), ErrorCode> {
        self.app_buffer_written.set(0);
        for process in self.grant.iter() {
            let process_id = process.processid();
            let started = process.enter(|grant, upcalls| {
                // If there's no buffer this means the process is dead, so
                // no need to issue a callback on this error case.
                let res: Result<(), ErrorCode> = grant
                    .buffer
                    .enter(|buffer| {
                        if let Some((algorithm, len)) = grant.request {
                            let copy_len = cmp::min(len, buffer.len());
                            if copy_len == 0 {
                                // 0-length or 0-size buffer
                                Err(ErrorCode::SIZE)
                            } else {
                                let res = self.crc.set_algorithm(algorithm);
                                match res {
                                    Ok(()) => {
                                        let copy_len = self.do_next_input(buffer, copy_len);
                                        if copy_len > 0 {
                                            self.app_buffer_written.set(copy_len);
                                            self.current_process.set(process_id);
                                            Ok(())
                                        } else {
                                            // Next input failed
                                            Err(ErrorCode::FAIL)
                                        }
                                    }
                                    Err(_) => {
                                        // Setting the algorithm failed
                                        Err(ErrorCode::INVAL)
                                    }
                                }
                            }
                        } else {
                            // no request
                            Err(ErrorCode::FAIL)
                        }
                    })
                    .unwrap_or(Err(ErrorCode::NOMEM));
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        if grant.request.is_some() {
                            upcalls
                                .schedule_upcall(
                                    0,
                                    kernel::errorcode::into_statuscode(Err(e)),
                                    0,
                                    0,
                                )
                                .ok();
                            grant.request = None;
                        }
                        Err(e)
                    }
                }
            });
            if started.is_ok() {
                return started;
            }
        }
        Err(ErrorCode::FAIL)
    }
}

/// Processes can use the Crc system call driver to compute Crc redundancy checks over process
/// memory.
///
/// At a high level, the client first provides a callback for the result of computations through
/// the `subscribe` system call and `allow`s the driver access to the buffer over-which to compute.
/// Then, it initiates a Crc computation using the `command` system call. See function-specific
/// comments for details.
impl<'a, C: Crc<'a>> SyscallDriver for CrcDriver<'a, C> {
    /// The `allow` syscall for this driver supports the single
    /// `allow_num` zero, which is used to provide a buffer over which
    /// to compute a Crc computation.
    ///
    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Provide user buffer to compute Crc over
            0 => self
                .grant
                .enter(process_id, |grant, _| {
                    mem::swap(&mut grant.buffer, &mut slice);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };
        if let Err(e) = res {
            Err((slice, e))
        } else {
            Ok(slice)
        }
    }

    // The `subscribe` syscall supports the single `subscribe_number`
    // zero, which is used to provide a callback that will receive the
    // result of a Crc computation.  The signature of the callback is
    //
    // ```
    //
    // fn callback(status: Result<(), ErrorCode>, result: usize) {}
    // ```
    //
    // where
    //
    //   * `status` is indicates whether the computation
    //     succeeded. The status `BUSY` indicates the unit is already
    //     busy. The status `SIZE` indicates the provided buffer is
    //     too large for the unit to handle.
    //
    //   * `result` is the result of the Crc computation when `status == BUSY`.
    //

    /// The command system call for this driver return meta-data about the driver and kicks off
    /// Crc computations returned through callbacks.
    ///
    /// ### Command Numbers
    ///
    ///   *   `0`: Returns non-zero to indicate the driver is present.
    ///
    ///   *   `1`: Requests that a Crc be computed over the buffer
    ///       previously provided by `allow`.  If none was provided,
    ///       this command will return `INVAL`.
    ///
    ///       This command's driver-specific argument indicates what Crc
    ///       algorithm to perform, as listed below.  If an invalid
    ///       algorithm specifier is provided, this command will return
    ///       `INVAL`.
    ///
    ///       If a callback was not previously registered with
    ///       `subscribe`, this command will return `INVAL`.
    ///
    ///       If a computation has already been requested by this
    ///       application but the callback has not yet been invoked to
    ///       receive the result, this command will return `BUSY`.
    ///
    ///       When `Ok(())` is returned, this means the request has been
    ///       queued and the callback will be invoked when the Crc
    ///       computation is complete.
    ///
    /// ### Algorithm
    ///
    /// The Crc algorithms supported by this driver are listed below.  In
    /// the values used to identify polynomials, more-significant bits
    /// correspond to higher-order terms, and the most significant bit is
    /// omitted because it always equals one.  All algorithms listed here
    /// consume each input byte from most-significant bit to
    /// least-significant.
    ///
    ///   * `0: Crc-32`  This algorithm is used in Ethernet and many other
    ///   applications.  It uses polynomial 0x04C11DB7 and it bit-reverses
    ///   and then bit-inverts the output.
    ///
    ///   * `1: Crc-32C`  This algorithm uses polynomial 0x1EDC6F41 (due
    ///   to Castagnoli) and it bit-reverses and then bit-inverts the
    ///   output.  It *may* be equivalent to various Crc functions using
    ///   the same name.
    ///
    ///   * `2: Crc-16CCITT`  This algorithm uses polynomial 0x1021 and does
    ///   no post-processing on the output value. The sixteen-bit Crc
    ///   result is placed in the low-order bits of the returned result
    ///   value. That is, result values will always be of the form `0x0000xxxx`
    ///   for this algorithm.  It can be performed purely in hardware on the SAM4L.
    fn command(
        &self,
        command_num: usize,
        algorithm_id: usize,
        length: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // This driver is present
            0 => CommandReturn::success(),

            // Request a Crc computation
            1 => {
                // Parse the user provided algorithm number
                let algorithm = if let Some(alg) = alg_from_user_int(algorithm_id) {
                    alg
                } else {
                    return CommandReturn::failure(ErrorCode::INVAL);
                };
                let res = self
                    .grant
                    .enter(process_id, |grant, _| {
                        if grant.request.is_some() {
                            Err(ErrorCode::BUSY)
                        } else if length > grant.buffer.len() {
                            Err(ErrorCode::SIZE)
                        } else {
                            grant.request = Some((algorithm, length));
                            Ok(())
                        }
                    })
                    .unwrap_or_else(|e| Err(ErrorCode::from(e)));

                match res {
                    Ok(()) => {
                        if self.current_process.is_none() {
                            self.next_request().map_or_else(
                                |e| CommandReturn::failure(ErrorCode::into(e)),
                                |_| CommandReturn::success(),
                            )
                        } else {
                            // Another request is ongoing. We've enqueued this one,
                            // wait for it to be started when it's its turn.
                            CommandReturn::success()
                        }
                    }
                    Err(e) => CommandReturn::failure(e),
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grant.enter(processid, |_, _| {})
    }
}

impl<'a, C: Crc<'a>> Client for CrcDriver<'a, C> {
    fn input_done(&self, result: Result<(), ErrorCode>, buffer: LeasableBuffer<'static, u8>) {
        // A call to `input` has finished. This can mean that either
        // we have processed the entire buffer passed in, or it was
        // truncated by the CRC unit as it was too large. In the first
        // case, we can see whether there is more outstanding data
        // from the app, whereas in the latter we need to advance the
        // LeasableBuffer window and pass it in again.
        let mut computing = false;
        // There are three outcomes to this match:
        //   - crc_buffer is not put back: input is ongoing
        //   - crc_buffer is put back and computing is true: compute is ongoing
        //   - crc_buffer is put back and computing is false: something failed, start a new request
        match result {
            Ok(()) => {
                // Completed leasable buffer, either refill it or compute
                if buffer.len() == 0 {
                    // Put the kernel buffer back
                    self.crc_buffer.replace(buffer.take());
                    self.current_process.map(|pid| {
                        let _res = self.grant.enter(*pid, |grant, upcalls| {
                            // This shouldn't happen unless there's a way to clear out a request
                            // through a system call: regardless, the request is gone, so cancel
                            // the CRC.
                            if grant.request.is_none() {
                                upcalls
                                    .schedule_upcall(
                                        0,
                                        kernel::errorcode::into_statuscode(Err(ErrorCode::FAIL)),
                                        0,
                                        0,
                                    )
                                    .ok();
                                return;
                            }

                            // Compute how many remaining bytes to compute over
                            let (alg, size) = grant.request.unwrap();
                            grant.request = Some((alg, size));
                            let size = cmp::min(size, grant.buffer.len());
                            // If the buffer has shrunk, size might be less than
                            // app_buffer_written: don't allow wraparound
                            let remaining = size - cmp::min(self.app_buffer_written.get(), size);

                            if remaining == 0 {
                                // No more bytes to input: compute
                                let res = self.crc.compute();
                                match res {
                                    Ok(()) => {
                                        computing = true;
                                    }
                                    Err(_) => {
                                        grant.request = None;
                                        upcalls
                                            .schedule_upcall(
                                                0,
                                                kernel::errorcode::into_statuscode(Err(
                                                    ErrorCode::FAIL,
                                                )),
                                                0,
                                                0,
                                            )
                                            .ok();
                                    }
                                }
                            } else {
                                // More bytes: do the next input
                                let amount = grant
                                    .buffer
                                    .enter(|app_slice| {
                                        self.do_next_input(
                                            &app_slice[self.app_buffer_written.get()..],
                                            remaining,
                                        )
                                    })
                                    .unwrap_or(0);
                                if amount == 0 {
                                    grant.request = None;
                                    upcalls
                                        .schedule_upcall(
                                            0,
                                            kernel::errorcode::into_statuscode(Err(
                                                ErrorCode::NOMEM,
                                            )),
                                            0,
                                            0,
                                        )
                                        .ok();
                                } else {
                                    self.app_buffer_written.add(amount);
                                }
                            }
                        });
                    });
                } else {
                    // There's more in the leasable buffer: pass it to input again
                    let res = self.crc.input(buffer);
                    match res {
                        Ok(()) => {}
                        Err((e, returned_buffer)) => {
                            self.crc_buffer.replace(returned_buffer.take());
                            self.current_process.map(|pid| {
                                let _res = self.grant.enter(*pid, |grant, upcalls| {
                                    grant.request = None;
                                    upcalls
                                        .schedule_upcall(
                                            0,
                                            kernel::errorcode::into_statuscode(Err(e)),
                                            0,
                                            0,
                                        )
                                        .ok();
                                });
                            });
                        }
                    }
                }
            }
            Err(e) => {
                // The callback returned an error, pass it back to userspace
                self.crc_buffer.replace(buffer.take());
                self.current_process.map(|pid| {
                    let _res = self.grant.enter(*pid, |grant, upcalls| {
                        grant.request = None;
                        upcalls
                            .schedule_upcall(0, kernel::errorcode::into_statuscode(Err(e)), 0, 0)
                            .ok();
                    });
                });
            }
        }
        // The buffer was put back (there is no input ongoing) but computing is false,
        // so no compute is ongoing. Start a new request if there is one.
        if self.crc_buffer.is_some() && !computing {
            let _ = self.next_request();
        }
    }

    fn crc_done(&self, result: Result<CrcOutput, ErrorCode>) {
        // First of all, inform the app about the finished operation /
        // the result
        self.current_process.take().map(|process_id| {
            let _ = self.grant.enter(process_id, |grant, upcalls| {
                grant.request = None;
                match result {
                    Ok(output) => {
                        let (val, user_int) = encode_upcall_crc_output(output);
                        upcalls
                            .schedule_upcall(
                                0,
                                kernel::errorcode::into_statuscode(Ok(())),
                                val as usize,
                                user_int as usize,
                            )
                            .ok();
                    }
                    Err(e) => {
                        upcalls
                            .schedule_upcall(0, kernel::errorcode::into_statuscode(Err(e)), 0, 0)
                            .ok();
                    }
                }
            });
        });
        let _ = self.next_request();
    }
}

fn alg_from_user_int(i: usize) -> Option<CrcAlgorithm> {
    match i {
        0 => Some(CrcAlgorithm::Crc32),
        1 => Some(CrcAlgorithm::Crc32C),
        2 => Some(CrcAlgorithm::Crc16CCITT),
        _ => None,
    }
}

fn encode_upcall_crc_output(output: CrcOutput) -> (u32, u32) {
    match output {
        CrcOutput::Crc32(val) => (val, 0),
        CrcOutput::Crc32C(val) => (val, 1),
        CrcOutput::Crc16CCITT(val) => (val as u32, 2),
    }
}
