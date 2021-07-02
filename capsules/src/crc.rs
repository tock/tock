//! Provides userspace access to a CRC unit.
//!
//! ## Instantiation
//!
//! Instantiate the capsule for use as a system call driver with a hardware
//! implementation and a `Grant` for the `App` type, and set the result as a
//! client of the hardware implementation. For example, using the SAM4L's `CRCU`
//! driver:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let crc_buffer = static_init!([u8; 64], [0; 64]);
//!
//! let crc = static_init!(
//!     capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
//!     capsules::crc::Crc::new(
//!         &mut sam4l::crccu::CRCCU,
//!         crc_buffer,
//!         board_kernel.create_grant(&grant_cap)
//!      )
//! );
//! sam4l::crccu::CRCCU.set_client(crc);
//!
//! ```
//!
//! ## CRC Algorithms
//!
//! The capsule supports two general purpose CRC algorithms, as well as a few
//! hardware specific algorithms implemented on the Atmel SAM4L.
//!
//! In the values used to identify polynomials below, more-significant bits
//! correspond to higher-order terms, and the most significant bit is omitted
//! because it always equals one.  All algorithms listed here consume each input
//! byte from most-significant bit to least-significant.
//!
//! ### CRC-32
//!
//! __Polynomial__: `0x04C11DB7`
//!
//! This algorithm is used in Ethernet and many other applications. It bit-
//! reverses and then bit-inverts the output.
//!
//! ### CRC-32C
//!
//! __Polynomial__: `0x1EDC6F41`
//!
//! Bit-reverses and then bit-inverts the output. It *may* be equivalent to
//! various CRC functions using the same name.
//!
//! ### SAM4L-16
//!
//! __Polynomial__: `0x1021`
//!
//! This algorithm does no post-processing on the output value. The sixteen-bit
//! CRC result is placed in the low-order bits of the returned result value, and
//! the high-order bits will all be set.  That is, result values will always be
//! of the form `0xFFFFxxxx` for this algorithm.  It can be performed purely in
//! hardware on the SAM4L.
//!
//! ### SAM4L-32
//!
//! __Polynomial__: `0x04C11DB7`
//!
//! This algorithm uses the same polynomial as `CRC-32`, but does no post-
//! processing on the output value.  It can be perfomed purely in hardware on
//! the SAM4L.
//!
//! ### SAM4L-32C
//!
//! __Polynomial__: `0x1EDC6F41`
//!
//! This algorithm uses the same polynomial as `CRC-32C`, but does no post-
//! processing on the output value.  It can be performed purely in hardware on
//! the SAM4L.

use core::cell::Cell;
use core::mem;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::hil::crc::{Crc, CrcAlgorithm, CrcClient, CrcOutput};
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId, Upcall};
use kernel::{Read, ReadOnlyAppSlice};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Crc as usize;

/// An opaque value maintaining state for one application's request
#[derive(Default)]
pub struct App {
    callback: Upcall,
    buffer: ReadOnlyAppSlice,
    waiting: Option<CrcAlgorithm>,
}

/// Struct that holds the state of the CRC driver and implements the `Driver` trait for use by
/// processes through the system call interface.
pub struct CrcDriver<'a, C: Crc<'a>> {
    crc_unit: &'a C,
    crc_buffer: TakeCell<'static, [u8]>,
    grant: Grant<App>,
    current_process: OptionalCell<ProcessId>,
    // We need to save (<how much we've already processed>, <how much we've copied into the LeasableBuffer>)
    app_buffer_progress: Cell<(usize, usize)>,
}

impl<'a, C: Crc<'a>> CrcDriver<'a, C> {
    /// Create a `Crc` driver
    ///
    /// The argument `crc_unit` must implement the abstract `CRC`
    /// hardware interface.  The argument `apps` should be an empty
    /// kernel `Grant`, and will be used to track application
    /// requests.
    ///
    /// ## Example
    ///
    /// ```rust
    /// capsules::crc::Crc::new(&sam4l::crccu::CRCCU, board_kernel.create_grant(&grant_cap));
    /// ```
    ///
    pub fn new(
        crc_unit: &'a C,
        crc_buffer: &'static mut [u8],
        grant: Grant<App>,
    ) -> CrcDriver<'a, C> {
        CrcDriver {
            crc_unit,
            crc_buffer: TakeCell::new(crc_buffer),
            grant,
            current_process: OptionalCell::empty(),
            app_buffer_progress: Cell::new((0, 0)),
        }
    }

    fn serve_current_process(&self) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}

/// Processes can use the CRC system call driver to compute CRC redundancy checks over process
/// memory.
///
/// At a high level, the client first provides a callback for the result of computations through
/// the `subscribe` system call and `allow`s the driver access to the buffer over-which to compute.
/// Then, it initiates a CRC computation using the `command` system call. See function-specific
/// comments for details.
impl<'a, C: Crc<'a>> Driver for CrcDriver<'a, C> {
    /// The `allow` syscall for this driver supports the single
    /// `allow_num` zero, which is used to provide a buffer over which
    /// to compute a CRC computation.
    ///
    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        let res = match allow_num {
            // Provide user buffer to compute CRC over
            0 => self
                .grant
                .enter(process_id, |grant| {
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

    /// The `subscribe` syscall supports the single `subscribe_number`
    /// zero, which is used to provide a callback that will receive the
    /// result of a CRC computation.  The signature of the callback is
    ///
    /// ```
    ///
    /// fn callback(status: Result<(), i2c::Error>, result: usize) {}
    /// ```
    ///
    /// where
    ///
    ///   * `status` is indicates whether the computation
    ///     succeeded. The status `BUSY` indicates the unit is already
    ///     busy. The status `SIZE` indicates the provided buffer is
    ///     too large for the unit to handle.
    ///
    ///   * `result` is the result of the CRC computation when `status == BUSY`.
    ///
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Upcall,
        process_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        let res = match subscribe_num {
            // Set callback for CRC result
            0 => self
                .grant
                .enter(process_id, |grant| {
                    mem::swap(&mut grant.callback, &mut callback);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        if let Err(e) = res {
            Err((callback, e))
        } else {
            Ok(callback)
        }
    }

    /// The command system call for this driver return meta-data about the driver and kicks off
    /// CRC computations returned through callbacks.
    ///
    /// ### Command Numbers
    ///
    ///   *   `0`: Returns non-zero to indicate the driver is present.
    ///
    ///   *   `2`: Requests that a CRC be computed over the buffer
    ///       previously provided by `allow`.  If none was provided,
    ///       this command will return `INVAL`.
    ///
    ///       This command's driver-specific argument indicates what CRC
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
    ///       queued and the callback will be invoked when the CRC
    ///       computation is complete.
    ///
    /// ### Algorithm
    ///
    /// The CRC algorithms supported by this driver are listed below.  In
    /// the values used to identify polynomials, more-significant bits
    /// correspond to higher-order terms, and the most significant bit is
    /// omitted because it always equals one.  All algorithms listed here
    /// consume each input byte from most-significant bit to
    /// least-significant.
    ///
    ///   * `0: CRC-32`  This algorithm is used in Ethernet and many other
    ///   applications.  It uses polynomial 0x04C11DB7 and it bit-reverses
    ///   and then bit-inverts the output.
    ///
    ///   * `1: CRC-32C`  This algorithm uses polynomial 0x1EDC6F41 (due
    ///   to Castagnoli) and it bit-reverses and then bit-inverts the
    ///   output.  It *may* be equivalent to various CRC functions using
    ///   the same name.
    ///
    ///   * `2: SAM4L-16`  This algorithm uses polynomial 0x1021 and does
    ///   no post-processing on the output value. The sixteen-bit CRC
    ///   result is placed in the low-order bits of the returned result
    ///   value, and the high-order bits will all be set.  That is, result
    ///   values will always be of the form `0xFFFFxxxx` for this
    ///   algorithm.  It can be performed purely in hardware on the SAM4L.
    ///
    ///   * `3: SAM4L-32`  This algorithm uses the same polynomial as
    ///   `CRC-32`, but does no post-processing on the output value.  It
    ///   can be perfomed purely in hardware on the SAM4L.
    ///
    ///   * `4: SAM4L-32C`  This algorithm uses the same polynomial as
    ///   `CRC-32C`, but does no post-processing on the output value.  It
    ///   can be performed purely in hardware on the SAM4L.
    fn command(
        &self,
        command_num: usize,
        algorithm_id: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // This driver is present
            0 => CommandReturn::success(),

            // Request a CRC computation
            2 => {
                // Parse the user provided algorithm number
                let algorithm = if let Some(alg) = alg_from_user_int(algorithm_id) {
                    alg
                } else {
                    return CommandReturn::failure(ErrorCode::INVAL);
                };

                // Check if there already is an operation in progress
                if self.current_process.is_some() {
                    // In that case, mark this process as waiting
                    self.grant
                        .enter(process_id, |grant| {
                            if grant.waiting.is_some() {
                                // Each app may make only one request at a time
                                CommandReturn::failure(ErrorCode::BUSY)
                            } else {
                                grant.waiting = Some(algorithm);
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|e| CommandReturn::failure(ErrorCode::from(e)))
                } else {
                    // We can start the operation immediately
                    self.current_process.set(process_id);
                    self.serve_current_process().map_or_else(
                        |e| CommandReturn::failure(ErrorCode::into(e)),
                        |_| CommandReturn::success(),
                    )
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}

impl<'a, C: Crc<'a>> CrcClient for CrcDriver<'a, C> {
    fn input_done(&self, result: Result<(), ErrorCode>, mut buffer: LeasableBuffer<'static, u8>) {
        // A call to `input` has finished. This can mean that either
        // we have processed the entire buffer passed in, or it was
        // truncated by the CRC unit as it was too large. In the first
        // case, we can see whether there is more outstanding data
        // from the app, whereas in the latter we need to advance the
        // LeasableBuffer window and pass it in again.

        unimplemented!()
    }

    fn crc_done(&self, result: Result<CrcOutput, ErrorCode>) {
        // First of all, inform the app about the finished operation /
        // the result
        self.current_process.take().map(|process_id| {
            let _ = self.grant.enter(process_id, |grant| {
                if let Ok(output) = result {
                    let (val, user_int) = encode_upcall_crc_output(output);
                    grant.callback.schedule(
                        kernel::into_statuscode(Ok(())),
                        val as usize,
                        user_int as usize,
                    );
                } else {
                    // TODO: Error handling
                }
            });
        });

        // Now that the CRC is finished, iterate through other apps
        // which are queued.
        //
        // TODO: implement fair queueing
        for process in self.grant.iter() {
            let process_id = process.processid();
            let started = process.enter(|grant| {
                if let Some(algorithm) = grant.waiting {
                    self.current_process.set(process_id);
                    self.serve_current_process();
                    true
                } else {
                    false
                }
            });

            // As soon as we have started an operation for an
            // additional process, break out of the loop
            if started {
                break;
            }
        }
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
        CrcOutput::Crc16CCITT(val) => ((val as u32) | 0xFFFF0000, 2),
    }
}
