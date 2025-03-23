// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Segger RTT implementation.
//!
//! RTT is a protocol for sending debugging messages to a connected host. The
//! embedded platform configures a portion of memory in a special way, and then
//! the host uses a JTAG connection to read the messages out of the chip's
//! memory.
//!
//! Receiving RTT Messages
//! ----------------------
//!
//! With the jlink tools, receiving RTT messages is a two step process. First,
//! open a JTAG connection with a command like:
//!
//! ```shell
//! $ JLinkExe -device nrf52 -if swd -speed 1000 -autoconnect 1
//! ```
//!
//! Then, use the `JLinkRTTClient` tool in a different terminal to print the
//! messages:
//!
//! ```shell
//! $ JLinkRTTClient
//! ```
//!
//! Todo
//! ----
//!
//! - Implement receive functionality.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! pub struct Platform {
//!     // Other fields omitted for clarity
//!     console: &'static capsules::console::Console<'static>,
//! }
//! ```
//!
//! In `main()`:
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! # use capsules::virtual_alarm::VirtualMuxAlarm;
//!
//! let virtual_alarm_rtt = static_init!(
//!     VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!     VirtualMuxAlarm::new(mux_alarm)
//! );
//! virtual_alarm_rtt.setup();
//!
//! let rtt_memory = static_init!(
//!     capsules::segger_rtt::SeggerRttMemory,
//!     capsules::segger_rtt::SeggerRttMemory::new(b"Terminal\0",
//!         &mut capsules::segger_rtt::UP_BUFFER,
//!         b"Terminal\0",
//!         &mut capsules::segger_rtt::DOWN_BUFFER)
//! );
//!
//! let rtt = static_init!(
//!     capsules::segger_rtt::SeggerRtt<VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>>,
//!     capsules::segger_rtt::SeggerRtt::new(virtual_alarm_rtt, rtt_memory,
//!         &mut capsules::segger_rtt::UP_BUFFER,
//!         &mut capsules::segger_rtt::DOWN_BUFFER)
//! );
//! virtual_alarm_rtt.set_client(rtt);
//!
//! let console = static_init!(
//!     capsules::console::Console<'static>,
//!     capsules::console::Console::new(
//!         rtt,
//!         &mut capsules::console::WRITE_BUF,
//!         &mut capsules::console::READ_BUF,
//!         board_kernel.create_grant(&grant_cap)
//!     )
//! );
//! kernel::hil::uart::UART::set_client(rtt, console);
//! console.initialize();
//! ```

use core::cell::Cell;
use core::marker::PhantomData;
use kernel::hil;
use kernel::hil::time::ConvertTicks;
use kernel::hil::uart;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;
use segger_unsafe::rtt_unsafe::{SeggerRttDownBuffer, SeggerRttUpBuffer};

/// Suggested length for the up buffer to pass to the Segger RTT capsule.
pub const DEFAULT_UP_BUFFER_LENGTH: usize = 1024;

/// Suggested length for the down buffer to pass to the Segger RTT capsule.
pub const DEFAULT_DOWN_BUFFER_LENGTH: usize = 32;

/// Milliseconds to wait to flush tx buffer after writing
const TX_MS_DELAY: u32 = 1;

/// Milliseconds to wait between checking if rx data is available
const RX_MS_DELAY: u32 = 100;

/// This structure is defined by the segger RTT protocol.
///
/// It must exist in memory in exactly this form so that the segger
/// JTAG tool can find it in the chip's memory and read and write
/// messages to the appropriate buffers.
///
/// Once this structure has been discovered by a debugger, portions
/// of the underlying memory are eligible to be written by external
/// host at any time, and can never be revoked. Similarly, the
/// configuration struct is expected by the host to be immutable
/// once discovered (which can happen any time the [id] string is
/// valid in-memory, which is written by the struct constructor).
/// For this reason, this struct must have a `'static` lifetime.
#[repr(C)]
pub struct SeggerRttMemory {
    id: [u8; 16],
    number_up_buffers: u32,
    number_down_buffers: u32,
    up_buffer: SeggerRttUpBuffer,
    down_buffer: SeggerRttDownBuffer,
    _lifetime: PhantomData<&'static ()>,
}

impl<'a> SeggerRttMemory {
    pub fn new_raw(
        up_buffer_name: &[u8],
        up_buffer: &mut [u8],
        down_buffer_name: &[u8],
        down_buffer: &'static [u8],
    ) -> SeggerRttMemory {
        SeggerRttMemory {
            // This field is a magic value that must be set to "SEGGER RTT" for the debugger to
            // recognize it when scanning the memory.
            //
            // In principle, there could be a risk that the value is duplicated elsewhere in
            // memory, therefore confusing the debugger. However in practice this hasn't caused any
            // known problem so far. If needed, this ID could be scrambled here, with the real magic
            // value being written only when this object is fully initialized.
            id: *b"SEGGER RTT\0\0\0\0\0\0",
            number_up_buffers: 1,
            number_down_buffers: 1,
            up_buffer: SeggerRttUpBuffer::new(up_buffer_name, up_buffer),
            down_buffer: SeggerRttDownBuffer::new(down_buffer_name, down_buffer),
            _lifetime: PhantomData,
        }
    }

    /// This getter allows access to the underlying buffer in the panic handler.
    /// The result is a pointer so that only `unsafe` code can actually dereference it - this is to
    /// restrict this priviledged access to the panic handler.
    pub fn get_up_buffer_ptr(&self) -> *const SeggerRttUpBuffer {
        &self.up_buffer
    }

    pub fn write_sync(&self, buf: &[u8]) {
        let mut ss = SubSlice::new(buf);
        while ss.len() > 0 {
            self.up_buffer.spin_until_sync();
            self.up_buffer.write_until_full(&mut ss);
        }
    }
}

pub struct SeggerRtt<'a, A: hil::time::Alarm<'a>> {
    alarm: &'a A, // Dummy alarm so we can get a callback.
    config: TakeCell<'static, SeggerRttMemory>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_client_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_client_buffer: MapCell<SubSliceMut<'static, u8>>,
}

impl<'a, A: hil::time::Alarm<'a>> SeggerRtt<'a, A> {
    pub fn new(alarm: &'a A, config: &'static mut SeggerRttMemory) -> SeggerRtt<'a, A> {
        SeggerRtt {
            alarm,
            config: TakeCell::new(config),
            tx_client: OptionalCell::empty(),
            tx_client_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            rx_client: OptionalCell::empty(),
            rx_client_buffer: MapCell::empty(),
        }
    }
}

impl<'a, A: hil::time::Alarm<'a>> uart::Transmit<'a> for SeggerRtt<'a, A> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.config.is_some() {
            self.config.map(|config| {
                let mut ss = SubSlice::new(tx_data);
                ss.slice(0..tx_len);
                config.up_buffer.write_until_full(&mut ss);

                self.tx_len.set(tx_len);
                // Save the client buffer so we can pass it back with the callback.
                self.tx_client_buffer.replace(tx_data);

                // Start a short timer so that we get a callback and can issue the callback to
                // the client.
                //
                // This heuristic interval was tested with the console capsule on a nRF52840-DK
                // board, passing buffers up to 1500 bytes from userspace. 100 micro-seconds
                // was too short, even for buffers as small as 128 bytes. 1 milli-second seems to
                // be reliable.
                let delay = self.alarm.ticks_from_ms(TX_MS_DELAY);
                self.alarm.set_alarm(self.alarm.now(), delay);
            });
            Ok(())
        } else {
            Err((ErrorCode::BUSY, tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::time::AlarmClient for SeggerRtt<'a, A> {
    fn alarm(&self) {
        self.tx_client.map(|client| {
            self.tx_client_buffer.take().map(|buffer| {
                client.transmitted_buffer(buffer, self.tx_len.get(), Ok(()));
            });
        });
        self.rx_client.map(|client| {
            if let Some(mut buffer) = self.rx_client_buffer.take() {
                self.config.map(|config| {
                    // Ask the down channel to read as many bytes as are
                    // available.
                    config.down_buffer.try_read(&mut buffer);
                });
                if buffer.len() == 0 {
                    // We've finished the requested read, reset to recover the
                    // length, and reliquensh the buffer.
                    buffer.reset();
                    let len = buffer.len();
                    client.received_buffer(buffer.take(), len, Ok(()), uart::Error::None);
                } else {
                    let delay = self.alarm.ticks_from_ms(RX_MS_DELAY);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                    self.rx_client_buffer.put(buffer)
                }
            };
        });
    }
}

// Dummy implementation so this can act as the underlying UART for a
// virtualized UART MUX. -pal 1/10/19
impl<'a, A: hil::time::Alarm<'a>> uart::Configure for SeggerRtt<'a, A> {
    fn configure(&self, _parameters: uart::Parameters) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a, A: hil::time::Alarm<'a>> uart::Receive<'a> for SeggerRtt<'a, A> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client)
    }

    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.rx_client_buffer
            .put(SubSliceMut::new(&mut buffer[0..len]));
        if !self.alarm.is_armed() {
            let delay = self.alarm.ticks_from_ms(RX_MS_DELAY);
            self.alarm.set_alarm(self.alarm.now(), delay);
        }
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
