// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022
// Copyright OxidOS Automotive SRL 2022
//
// Author: Teona Severin <teona.severin@oxidos.io>

//! Interface for CAN peripherals.
//!
//! Defines multiple traits for different purposes.
//!
//! The `Configure` trait is used to configure the communication
//! mode and bit timing parameters of the CAN peripheral. The
//! `ConfigureFd` trait is an advanced feature that can be implemented
//! for peripherals that support flexible data messages. These
//! 2 traits represent synchronous actions and do not need a client
//! in order to confirm to the capsule that the action is finished.
//!
//! The `Controller` trait is used to enable and disable the device.
//! In order to be able to enable the device, the bit timing parameters
//! and the communication mode must be previously set, and in order
//! to disable the device, it must be enabled. This trait defines
//! asynchronous behaviours and because of that, the `ControllerClient`
//! trait is used to confirm to the capsule that the action is finished.
//!
//! The `Filter` trait is used to configure filter banks for receiving
//! messages. The action is synchronous.
//!
//! The `Transmit` trait is used to asynchronously send a message on
//! the CAN bus. The device must be previously enabled. The
//! `TransmitClient` trait is used to notify the capsule when the
//! transmission is done or when there was en error captured during
//! the transmission.
//!
//! The `Receive` trait is used to asynchronously receive messages on
//! the CAN bus. The `ReceiveClient` trait is used to notify the capsule
//! when a message was received, when the receiving process was aborted
//! and anytime an error occurs.
//!

use crate::ErrorCode;
use core::cmp;

pub const STANDARD_CAN_PACKET_SIZE: usize = 8;
pub const FD_CAN_PACKET_SIZE: usize = 64;

/// Defines the possible states of the peripheral
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    /// The peripheral is enabled and functions normally
    Running,

    /// The peripheral is disabled
    Disabled,

    /// There was an error while executing a request (sending
    /// or receiving)
    Error(Error),
}

/// Defines the error codes received from the CAN peripheral
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// The previous transmission failed due to an arbitration
    /// lost
    ArbitrationLost,

    /// The previous transmission failed
    Transmission,

    /// The internal Transmit Error Counter or the internal
    /// Receive Error Counter is greater than 96 and the
    /// passive error state is entered.
    Warning,

    /// The internal Transmit Error Counter or the internal
    /// Receive Error Counter is greater than 127 and the
    /// passive error state is entered.
    Passive,

    /// The internal Transmit Error Counter is greater than 255
    /// and the bus-off state is entered.
    BusOff,

    /// 6 consecutive bits of equal value are detected on the bus.
    /// (When transmitting device detects 5 consecutive bits of
    /// equal value on the bus, it inserts a complemented one).
    Stuff,

    /// The form of the received or the transmitted frame is
    /// different than the standard format.
    Form,

    /// There are no receivers on the bus or the sender caused an
    /// error.
    Ack,

    /// While transmitting a recessive bit, the receiver sensed a
    /// dominant bit.
    BitRecessive,

    /// While transmitting a dominant bit, the receiver sensed a
    /// recessive bit.
    BitDominant,

    /// The frame has been corrupted on the CAN bus
    Crc,

    /// Set by software to force the hardware to indicate the
    /// current communication status.
    SetBySoftware,
}

impl From<Error> for ErrorCode {
    fn from(val: Error) -> Self {
        match val {
            Error::ArbitrationLost => ErrorCode::RESERVE,
            Error::BusOff => ErrorCode::OFF,
            Error::Form => ErrorCode::INVAL,
            Error::BitRecessive | Error::BitDominant => ErrorCode::BUSY,
            Error::Ack | Error::Transmission => ErrorCode::NOACK,
            Error::Crc | Error::SetBySoftware | Error::Warning | Error::Passive | Error::Stuff => {
                ErrorCode::FAIL
            }
        }
    }
}

/// The Scale Bits structure defines the 2 possible widths
/// of the filter bank
#[derive(Debug, Copy, Clone)]
pub enum ScaleBits {
    Bits16,
    Bits32,
}

/// The filter can be configured to filter the messages by matching
/// an identifier or by bitwise matching multiple identifiers.
#[derive(Debug, Copy, Clone)]
pub enum IdentifierMode {
    /// A mask is used to filter the messages
    List,
    /// The value of the identifier is used to filter the messages
    Mask,
}

/// The identifier can be standard (11 bits) or extended (29 bits)
#[derive(Debug, Copy, Clone)]
pub enum Id {
    Standard(u16),
    Extended(u32),
}

/// This structure defines the parameters to configure a filter bank
#[derive(Copy, Clone)]
pub struct FilterParameters {
    /// The filter Id
    ///
    /// This value is dependent on the peripheral used and identifies
    /// the filter bank that will be used
    pub number: u32,

    /// The width of the filter bank
    pub scale_bits: ScaleBits,

    /// The way in which the message Ids will be filtered.
    pub identifier_mode: IdentifierMode,

    /// The receive FIFO Id that the filter will be applied to
    pub fifo_number: usize,
}

/// This structure defines the parameters for the timing mode
#[derive(Debug, Copy, Clone)]
pub struct BitTiming {
    /// A value that defines the location of the sample
    /// point (between 1 and 16 time quanta)
    pub segment1: u8,

    /// A value that defines the location of the transmit
    /// point (between 1 and 8 time quanta)
    pub segment2: u8,

    /// A value used for compensating the delay on the bus
    /// lines
    pub propagation: u8,

    /// A value that represents the maximum time by which
    /// the bit sampling period may lengthen or shorten
    /// each cycle to perform the resynchronization. It is
    /// measured in time quanta.
    pub sync_jump_width: u32,

    /// A value that represents the sampling clock period.
    /// A period is reffered to as a time quanta.
    pub baud_rate_prescaler: u32,
}

/// The peripheral can be configured to work in the following modes:
#[derive(Debug, Copy, Clone)]
pub enum OperationMode {
    /// Loopback mode means that each message is transmitted on the
    /// TX channel and immediately received on the RX channel
    Loopback,

    /// Monitoring mode means that the CAN peripheral sends only the recessive
    /// bits on the bus and cannot start a transmission, but can receive
    /// valid data frames and valid remote frames
    Monitoring,

    /// Freeze mode means that no transmission or reception of frames is
    /// done
    Freeze,

    /// Normal mode means that the transmission and reception of frames
    /// are available
    Normal,
}

/// The `StandardBitTiming` trait is used to calculate the optimum timing parameters
/// for a given bitrate and the clock's frequency.
pub trait StandardBitTiming {
    fn bit_timing_for_bitrate(clock_rate: u32, bitrate: u32) -> Result<BitTiming, ErrorCode>;
}

/// The default implementation for the `bit_timing_for_bitrate` method. This algorithm
/// is inspired by the Zephyr CAN driver available at
/// `<https://github.com/zephyrproject-rtos/zephyr/tree/main/drivers/can>`
impl<T: Configure> StandardBitTiming for T {
    fn bit_timing_for_bitrate(clock_rate: u32, bitrate: u32) -> Result<BitTiming, ErrorCode> {
        if bitrate > 8_000_000 {
            return Err(ErrorCode::INVAL);
        }

        let mut res_timing: BitTiming = Self::MIN_BIT_TIMINGS;
        let sp: u32 = if bitrate > 800_000 {
            750
        } else if bitrate > 500_000 {
            800
        } else {
            875
        };
        let mut sample_point_err;
        let mut sample_point_err_min = u16::MAX;
        let mut ts: u32 = (Self::MAX_BIT_TIMINGS.propagation
            + Self::MAX_BIT_TIMINGS.segment1
            + Self::MAX_BIT_TIMINGS.segment2
            + Self::SYNC_SEG) as u32;

        for prescaler in
            cmp::max(clock_rate / (ts * bitrate), 1)..Self::MAX_BIT_TIMINGS.baud_rate_prescaler
        {
            if clock_rate % (prescaler * bitrate) != 0 {
                continue;
            }
            ts = clock_rate / (prescaler * bitrate);

            sample_point_err = {
                let ts1_max = Self::MAX_BIT_TIMINGS.propagation + Self::MAX_BIT_TIMINGS.segment1;
                let ts1_min = Self::MIN_BIT_TIMINGS.propagation + Self::MIN_BIT_TIMINGS.segment1;
                let mut ts1;
                let mut ts2;
                let mut res: i32 = 0;

                ts2 = ts - (ts * sp) / 1000;
                ts2 = if ts2 < Self::MIN_BIT_TIMINGS.segment2 as u32 {
                    Self::MIN_BIT_TIMINGS.segment2 as u32
                } else if ts2 > Self::MAX_BIT_TIMINGS.segment2 as u32 {
                    Self::MAX_BIT_TIMINGS.segment2 as u32
                } else {
                    ts2
                };
                ts1 = ts - Self::SYNC_SEG as u32 - ts2;

                if ts1 > ts1_max as u32 {
                    ts1 = ts1_max as u32;
                    ts2 = ts - Self::SYNC_SEG as u32 - ts1;
                    if ts2 > Self::MAX_BIT_TIMINGS.segment2 as u32 {
                        res = -1;
                    }
                } else if ts1 < ts1_min as u32 {
                    ts1 = ts1_min as u32;
                    ts2 = ts - ts1;
                    if ts2 < Self::MIN_BIT_TIMINGS.segment2 as u32 {
                        res = -1;
                    }
                }

                if res != -1 {
                    res_timing.propagation = if ts1 / 2 < Self::MIN_BIT_TIMINGS.propagation as u32 {
                        Self::MIN_BIT_TIMINGS.propagation
                    } else if ts1 / 2 > Self::MAX_BIT_TIMINGS.propagation as u32 {
                        Self::MAX_BIT_TIMINGS.propagation
                    } else {
                        (ts1 / 2) as u8
                    };

                    res_timing.segment1 = ts1 as u8 - res_timing.propagation;
                    res_timing.segment2 = ts2 as u8;

                    res = ((Self::SYNC_SEG as u32 + ts1) * 1000 / ts) as i32;
                    if res > sp as i32 {
                        res - sp as i32
                    } else {
                        sp as i32 - res
                    }
                } else {
                    res
                }
            };

            if sample_point_err < 0 {
                continue;
            }

            if sample_point_err < sample_point_err_min as i32 {
                sample_point_err_min = sample_point_err as u16;
                res_timing.baud_rate_prescaler = prescaler;
                if sample_point_err == 0 {
                    break;
                }
            }
        }

        if sample_point_err_min != 0 {
            return Err(ErrorCode::INVAL);
        }

        Ok(BitTiming {
            segment1: res_timing.segment1 - 1,
            segment2: res_timing.segment2 - 1,
            propagation: res_timing.propagation,
            sync_jump_width: if res_timing.sync_jump_width == 0 {
                0
            } else {
                res_timing.sync_jump_width - 1
            },
            baud_rate_prescaler: res_timing.baud_rate_prescaler - 1,
        })
    }
}

/// The `Configure` trait is used to configure the CAN peripheral and to prepare it for
/// transmission and reception of data. The peripheral cannot transmit or receive frames if
/// it is not previously configured and enabled.
///
/// In order to configure the peripheral, the following steps are required:
///
/// - Call `set_bitrate` or `set_bit_timing` to configure the timing settings
/// - Call `set_operation_mode` to configure the testing mode
/// - (Optional) Call `set_automatic_retransmission` and/or
///   `set_wake_up` to configure the behaviour of the peripheral
/// - To apply the settings and be able to use the peripheral, call `enable`
///   (from the `Controller` trait)
pub trait Configure {
    /// Constants that define the minimum and maximum values that the timing
    /// parameters can take. They are used when calculating the optimum timing
    /// parameters for a given bitrate.
    const MIN_BIT_TIMINGS: BitTiming;
    const MAX_BIT_TIMINGS: BitTiming;

    /// This constant represents the synchronization segment.
    /// Most CAN devices seems to have define this in hardware to 1 quantum long.
    /// 1 quantum long. It is used for the synchronization of the clocks.
    const SYNC_SEG: u8 = 1;

    /// Configures the CAN peripheral at the given bitrate. This function is
    /// supposed to be called before the `enable` function. This function is
    /// synchronous as the driver should only calculate the timing parameters
    /// based on the bitrate and the frequency of the board and store them.
    /// This function does not configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `bitrate` - A value that represents the bitrate for the CAN communication.
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The timing parameters were calculated and stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    fn set_bitrate(&self, bitrate: u32) -> Result<(), ErrorCode>;

    /// Configures the CAN peripheral with the given arguments. This function is
    /// supposed to be called before the `enable` function. This function is
    /// synchronous as the driver should only store the arguments, and should not
    /// configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `bit_timing` - A BitTiming structure to define the bit timing
    ///                  settings for the peripheral
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The parameters were stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    fn set_bit_timing(&self, bit_timing: BitTiming) -> Result<(), ErrorCode>;

    /// Configures the CAN peripheral with the given arguments. This function is
    /// supposed to be called before the `enable` function. This function is
    /// synchronous as the driver should only store the arguments, and should not
    /// configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `mode` - An OperationMode structure to define the running mode
    ///            of the peripheral
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The parameters were stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    fn set_operation_mode(&self, mode: OperationMode) -> Result<(), ErrorCode>;

    /// Returns the current timing parameters for the CAN peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(BitTiming)` - The current timing parameters
    ///                            given to the peripheral
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    fn get_bit_timing(&self) -> Result<BitTiming, ErrorCode>;

    /// Returns the current operating mode for the CAN peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(OperationMode)` - The current operating mode parameter
    ///                         given to the peripheral
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    fn get_operation_mode(&self) -> Result<OperationMode, ErrorCode>;

    /// Configures the CAN peripheral with the automatic retransmission setting.
    /// This function is optional, but if used, must be called before the
    /// `enable` function. This function is synchronous as the driver should
    /// only store the arguments, and should not configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `automatic` - Value to configure the automatic retransmission
    ///                 setting
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The setting was stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    fn set_automatic_retransmission(&self, automatic: bool) -> Result<(), ErrorCode>;

    /// Configures the CAN peripheral with the automatic wake up setting.
    /// This function is optional, but if used, must be called before the
    /// `enable` function. This function is synchronous as the driver should
    /// only store the arguments, and should not configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `wake_up` - Value to configure the automatic wake up setting
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The setting was stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    fn set_wake_up(&self, wake_up: bool) -> Result<(), ErrorCode>;

    /// Returns the current automatic retransmission setting of the peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(bool)` - The current automatic retransmission setting
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    fn get_automatic_retransmission(&self) -> Result<bool, ErrorCode>;

    /// Returns the current automatic wake up setting of the peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(bool)` - The current automatic wake up setting
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    fn get_wake_up(&self) -> Result<bool, ErrorCode>;

    /// Returns the number of receive FIFOs the peripheral provides
    fn receive_fifo_count(&self) -> usize;
}

/// The `ConfigureFd` trait is used to configure the CAN peripheral for CanFD and to prepare it for
/// transmission and reception of data. The peripheral cannot transmit or receive frames if
/// it is not previously configured and enabled.
///
/// In order to configure the peripheral, the following steps are required:
///
/// - Call `set_bit_timing` to configure the timing settings
/// - Call `set_operation_mode` to configure the testing mode
/// - (Optional) Call `set_automatic_retransmission` and/or
///   `set_wake_up` to configure the behaviour of the peripheral
/// - To apply the settings and be able to use the peripheral, call `enable`
///   (from the `Controller` trait)
pub trait ConfigureFd: Configure {
    /// Configures the CAN FD peripheral with the given arguments. This function is
    /// supposed to be called before the `enable` function. This function is
    /// synchronous as the driver should only store the arguments, and should not
    /// configure the hardware.
    ///
    /// # Arguments:
    ///
    /// * `payload_bit_timing` - A BitTiming structure to define the bit timing
    ///                         settings for the frame payload
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The parameters were stored.
    /// * `Err(ErrorCode)` - Indicates the error because of which the request
    ///                      cannot be completed
    ///                    - `ErrorCode::NOSUPPORT` indicates that payload timing
    ///                      is not supported
    fn set_payload_bit_timing(&self, payload_bit_timing: BitTiming) -> Result<(), ErrorCode>;

    /// Returns the current timing parameters for the CAN peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(BitTiming)` - The current timing for the frame payload
    ///                     given to the peripheral
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    ///                    - `ErrorCode::NOSUPPORT` indicates that payload timing
    ///                      is not supported
    fn get_payload_bit_timing(&self) -> Result<BitTiming, ErrorCode>;

    /// Returns the maximum accepted frame size in bytes.
    ///
    /// - for CanFD BRS this should be 8 bytes
    /// - for CanFD Full this should be 64 bytes
    fn get_frame_size() -> usize;
}

/// The `Filter` trait is used to enable and disable a filter bank.
///
/// When the receiving process starts by calling the `start_receiving_process`
/// in the `Receive` trait, there MUST be no filter enabled.
pub trait Filter {
    /// Enables a filter for message reception.
    ///
    /// # Arguments:
    ///
    /// * `filter` - A FilterParameters structure to define the filter
    ///                  configuration
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The filter was successfully configured.
    /// * `Err(ErrorCode)` - indicates the error because of which the
    ///                      request cannot be completed
    fn enable_filter(&self, filter: FilterParameters) -> Result<(), ErrorCode>;

    /// Disables a filter.
    ///
    /// # Arguments:
    ///
    /// * `number` - The filter Id to identify the filter bank
    ///                     to disable
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The filter was successfully disabled.
    /// * `Err(ErrorCode)` - indicates the error because of which the
    ///                      request cannot be completed
    fn disable_filter(&self, number: u32) -> Result<(), ErrorCode>;

    /// Returns the number of filters the peripheral provides
    fn filter_count(&self) -> usize;
}

/// The `Controller` trait is used to enable and disable the CAN peripheral.
/// The enable process applies the settings that were previously provided
/// to the driver using the `Configure` trait.
pub trait Controller {
    /// Set the client to be used for callbacks of the `Controller` implementation.
    fn set_client(&self, client: Option<&'static dyn ControllerClient>);

    /// This function enables the CAN peripheral with the Timing, Operation and Mode
    /// arguments that are provided to the driver before calling the
    /// `enable` function.
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The parameters were provided and the process can begin.
    ///            The driver will call the `state_changed` and `enabled`
    ///            callbacks after the process ends. Both of the callbacks
    ///            must be called and the capsule should wait for the `enable`
    ///            callback before transmitting or receiving frames, as enabling
    ///            might fail with an error. While `state_changed` will report
    ///            the device as being in `State::Disabled`, it does not report
    ///            the error. A client cannot otherwise differentiate between
    ///            a callback issued due to failed `enable` or a peripheral's decision
    ///            to enter a disabled state.
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed.
    ///     * `ErrorCode::BUSY` - the peripheral was already enabled
    ///     * `ErrorCode::INVAL` - no arguments were previously provided
    fn enable(&self) -> Result<(), ErrorCode>;

    /// This function disables the CAN peripheral and puts it in Sleep Mode. The
    /// peripheral must be previously enabled.
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The peripheral was already enabled and the process can begin.
    ///            The driver will call the `state_changed` and `disabled`
    ///            callbacks after the process ends. Both of the callbacks
    ///            must be called and the capsule should wait for the `disabled`
    ///            callback before considering the peripheral disabled, as disabling
    ///            might fail with an erro . While `state_changed` will report
    ///            the device as being in `State::Enabled`, it does not report
    ///            the error. A client cannot otherwise differentiate between
    ///            a callback issued due to failed `disable` or a peripheral's decision
    ///            to enter the enable state.
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed.
    ///     * `ErrorCode::OFF` - the peripheral was not previously enabled
    fn disable(&self) -> Result<(), ErrorCode>;

    /// This function returns the current state of the CAN peripheral.
    ///
    /// # Return values:
    ///
    /// * `Ok(State)` - The state of the CAN peripheral if it is functional
    /// * `Err(ErrorCode)` - The driver cannot report the state of the peripheral
    ///                      if it is not functional.
    fn get_state(&self) -> Result<State, ErrorCode>;
}

/// The `Transmit` trait is used to interact with the CAN driver through transmission
/// requests only.
///
/// The CAN peripheral must be configured first, in order to be able to send data.
pub trait Transmit<const PACKET_SIZE: usize> {
    const PACKET_SIZE: usize = PACKET_SIZE;
    /// Set the client to be used for callbacks of the `Transmit` implementation.
    fn set_client(&self, client: Option<&'static dyn TransmitClient<PACKET_SIZE>>);

    /// Sends a buffer using the CAN bus.
    ///
    /// In most cases, this function should be called after the peripheral was
    /// previously configures and at least one filter has been enabled.
    ///
    /// # Arguments:
    ///
    /// * `id` - The identifier of the message (standard or extended)
    /// * `buffer` - Data to be written on the bus
    /// * `len` - Length of the current message
    ///
    /// # Return values:
    /// * `Ok()` - The transmission request was successful and the caller
    ///            will receive a for the `transmit_complete` callback function call
    /// * `Err(ErrorCode, &'static mut [u8])` - a tuple with the error that occurred
    ///                                         during the transmission request and
    ///                                         the buffer that was provided as an
    ///                                         argument to the function
    fn send(
        &self,
        id: Id,
        buffer: &'static mut [u8; PACKET_SIZE],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8; PACKET_SIZE])>;
}

/// The `Receive` trait is used to interact with the CAN driver through receive
/// requests only.
///
/// The CAN peripheral must be configured first, in order to be able to send data.
pub trait Receive<const PACKET_SIZE: usize> {
    const PACKET_SIZE: usize = PACKET_SIZE;
    /// Set the client to be used for callbacks of the `Receive` implementation.
    fn set_client(&self, client: Option<&'static dyn ReceiveClient<PACKET_SIZE>>);

    /// Start receiving messaged on the CAN bus.
    ///
    /// In most cases, this function should be called after the peripheral was
    /// previously configured. When calling this function, there MUST be
    /// no filters enabled by the user. The implementation of this function
    /// MUST permit receiving frames on all available receiving FIFOs.
    ///
    /// # Arguments:
    ///
    /// * `buffer` - A buffer to store the data
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The receive request was successful and the caller waits for the
    ///            `message_received` callback function to receive data
    /// * `Err(ErrorCode, &'static mut [u8])` - tuple with the error that occurred
    ///                                         during the reception request and
    ///                                         the buffer that was received as an
    ///                                         argument to the function
    fn start_receive_process(
        &self,
        buffer: &'static mut [u8; PACKET_SIZE],
    ) -> Result<(), (ErrorCode, &'static mut [u8; PACKET_SIZE])>;

    /// Asks the driver to stop receiving messages. This function should
    /// be called only after a call to the `start_receive_process` function.
    ///
    /// # Return values:
    ///
    /// * `Ok()` - The request was successful an the caller waits for the
    ///            `stopped` callback function after this command
    /// * `Err(ErrorCode)` - Indicates the error because of which the
    ///                      request cannot be completed
    fn stop_receive(&self) -> Result<(), ErrorCode>;
}

/// Client interface for capsules that implement the `Controller` trait.
pub trait ControllerClient {
    /// The driver calls this function when the state of the CAN peripheral is
    /// changed.
    ///
    /// # Arguments:
    ///
    /// * `state` - The current state of the peripheral
    fn state_changed(&self, state: State);

    /// The driver calls this function when the peripheral has been successfully
    /// enabled. The driver must call this function and `state_changed` also,
    /// but must wait for this function to be called. If an error occurs, the
    /// `state_changed` callback might not be able to report it.
    ///
    /// # Arguments:
    ///
    /// * `status`
    ///     * `Ok()` - The peripheral has been successfully enabled; the
    ///                actual state is transmitted via `state_changed` callback
    ///     * `Err(ErrorCode)` - The error that occurred during the enable process
    fn enabled(&self, status: Result<(), ErrorCode>);

    /// The driver calls this function when the peripheral has been successfully
    /// disabled. The driver must call this function and `state_changed` also,
    /// but must wait for this function to be called. If an error occurs, the
    /// `state_changed` callback might not be able to report it.
    ///
    /// # Arguments:
    ///
    /// * `status`
    ///     * `Ok()` - The peripheral has been successfully disabled; the
    ///                actual state is transmitted via `state_changed` callback
    ///     * `Err(ErrorCode)` - The error that occurred during the disable process
    fn disabled(&self, status: Result<(), ErrorCode>);
}

/// Client interface for capsules that implement the `Transmit` trait.
pub trait TransmitClient<const PACKET_SIZE: usize> {
    /// The driver calls this function when there is an update of the last
    /// message that was transmitted
    ///
    /// # Arguments:
    ///
    /// * `status` - The status for the request
    ///     * `Ok()` - There was no error during the transmission process
    ///     * `Err(Error)` - The error that occurred during the transmission process
    /// * `buffer` - The buffer received as an argument for the `send` function
    fn transmit_complete(&self, status: Result<(), Error>, buffer: &'static mut [u8; PACKET_SIZE]);
}

/// Client interface for capsules that implement the `Receive` trait.
pub trait ReceiveClient<const PACKET_SIZE: usize> {
    /// The driver calls this function when a new message has been received on the given
    /// FIFO.
    ///
    /// # Arguments:
    ///
    /// * `id` - The identifier of the received message
    /// * `buffer` - A reference to the buffer where the data is stored. This data must
    ///              be stored. This buffer is usually a slice to the original buffer
    ///              that was supplied to the `start_receive_process`. It must be used
    ///              within this function call. In most cases the data is copied to a
    ///              driver or application buffer.
    /// * `len` - The length of the buffer
    /// * `status` - The status for the request
    ///     * `Ok()` - There was no error during the reception process
    ///     * `Err(Error)` - The error that occurred during the reception process
    fn message_received(
        &self,
        id: Id,
        buffer: &mut [u8; PACKET_SIZE],
        len: usize,
        status: Result<(), Error>,
    );

    /// The driver calls this function when the reception of messages has been stopped.
    ///
    /// # Arguments:
    ///
    /// * `buffer` - The buffer that was given as an argument to the
    ///               `start_receive_process` function
    fn stopped(&self, buffer: &'static mut [u8; PACKET_SIZE]);
}

/// Convenience type for capsules that configure, send
/// and receive data using the CAN peripheral
pub trait Can:
    Transmit<STANDARD_CAN_PACKET_SIZE> + Configure + Controller + Receive<STANDARD_CAN_PACKET_SIZE>
{
}

pub trait CanFd:
    Transmit<FD_CAN_PACKET_SIZE> + Configure + ConfigureFd + Receive<FD_CAN_PACKET_SIZE>
{
}

/// Provide blanket implementation for Can trait group
impl<
        T: Transmit<STANDARD_CAN_PACKET_SIZE>
            + Configure
            + Controller
            + Receive<STANDARD_CAN_PACKET_SIZE>,
    > Can for T
{
}

/// Provide blanket implementation for CanFd trait group
impl<T: Transmit<FD_CAN_PACKET_SIZE> + Configure + ConfigureFd + Receive<FD_CAN_PACKET_SIZE>> CanFd
    for T
{
}
