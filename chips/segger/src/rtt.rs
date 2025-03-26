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
use core::ops::Index;
use core::sync::atomic::{fence, Ordering};
use kernel::hil;
use kernel::hil::time::ConvertTicks;
use kernel::hil::uart;
use kernel::utilities::cells::{OptionalCell, TakeCell, VolatileCell};
use kernel::ErrorCode;

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
#[repr(C)]
pub struct SeggerRttMemory<'a> {
    id: [u8; 16],
    number_up_buffers: u32,
    number_down_buffers: u32,
    up_buffer: SeggerRttBuffer<'a>,
    down_buffer: SeggerRttBuffer<'a>,
}

#[repr(C)]
pub struct SeggerRttBuffer<'a> {
    name: *const u8, // Pointer to the name of this channel. Must be a 4 byte thin pointer.
    // These fields are marked as `pub` to allow access in the panic handler.
    pub buffer: *const VolatileCell<u8>, // Pointer to the buffer for this channel.
    pub length: u32,
    pub write_position: VolatileCell<u32>,
    read_position: VolatileCell<u32>,
    flags: u32,
    _lifetime: PhantomData<&'a ()>,
}

impl Index<usize> for SeggerRttBuffer<'_> {
    type Output = VolatileCell<u8>;

    fn index(&self, index: usize) -> &Self::Output {
        let index = index as isize;
        if index >= self.length as isize {
            panic!("Index out of bounds {}/{}", index, self.length)
        } else {
            unsafe { &*self.buffer.offset(index) }
        }
    }
}

impl<'a> SeggerRttMemory<'a> {
    pub fn new_raw(
        up_buffer_name: &'a [u8],
        up_buffer: &'a [VolatileCell<u8>],
        down_buffer_name: &'a [u8],
        down_buffer: &'a [VolatileCell<u8>],
    ) -> SeggerRttMemory<'a> {
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
            up_buffer: SeggerRttBuffer {
                name: up_buffer_name.as_ptr(),
                buffer: up_buffer.as_ptr(),
                length: up_buffer.len() as u32,
                write_position: VolatileCell::new(0),
                read_position: VolatileCell::new(0),
                flags: 0,
                _lifetime: PhantomData,
            },
            down_buffer: SeggerRttBuffer {
                name: down_buffer_name.as_ptr(),
                buffer: down_buffer.as_ptr(),
                length: down_buffer.len() as u32,
                write_position: VolatileCell::new(0),
                read_position: VolatileCell::new(0),
                flags: 0,
                _lifetime: PhantomData,
            },
        }
    }

    /// This getter allows access to the underlying buffer in the panic handler.
    /// The result is a pointer so that only `unsafe` code can actually dereference it - this is to
    /// restrict this priviledged access to the panic handler.
    pub fn get_up_buffer_ptr(&self) -> *const SeggerRttBuffer<'a> {
        &self.up_buffer
    }

    pub fn write_sync(&self, buf: &[u8]) {
        let mut index = self.up_buffer.write_position.get() as usize;
        fence(Ordering::SeqCst);

        let buffer_len = self.up_buffer.length as usize;
        for c in buf.iter() {
            index = (index + 1) % buffer_len;
            while self.up_buffer.read_position.get() as usize == index {
                core::hint::spin_loop();
            }
            self.up_buffer[index].set(*c);
            fence(Ordering::SeqCst);
            self.up_buffer.write_position.set(index as u32);
            fence(Ordering::SeqCst);
        }
    }
}

pub struct SeggerRtt<'a, A: hil::time::Alarm<'a>> {
    alarm: &'a A, // Dummy alarm so we can get a callback.
    config: TakeCell<'a, SeggerRttMemory<'a>>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    tx_client_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_client_buffer: TakeCell<'static, [u8]>,
    rx_cursor: Cell<usize>,
    rx_len: Cell<usize>,
}

impl<'a, A: hil::time::Alarm<'a>> SeggerRtt<'a, A> {
    pub fn new(alarm: &'a A, config: &'a mut SeggerRttMemory<'a>) -> SeggerRtt<'a, A> {
        SeggerRtt {
            alarm,
            config: TakeCell::new(config),
            tx_client: OptionalCell::empty(),
            tx_client_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            rx_client: OptionalCell::empty(),
            rx_client_buffer: TakeCell::empty(),
            rx_cursor: Cell::new(0),
            rx_len: Cell::new(0),
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
                // Copy the incoming data into the buffer. Once we increment
                // the `write_position` the RTT listener will go ahead and read
                // the message from us.
                let mut index = config.up_buffer.write_position.get() as usize;
                fence(Ordering::SeqCst);

                let buffer_len = config.up_buffer.length as usize;
                for i in 0..tx_len {
                    config.up_buffer[(i + index) % buffer_len].set(tx_data[i]);
                }
                fence(Ordering::SeqCst);

                index = (index + tx_len) % buffer_len;
                config.up_buffer.write_position.set(index as u32);
                fence(Ordering::SeqCst);

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
            self.rx_client_buffer.take().map(|buffer| {
                self.config.map(|config| {
                    let write_position = &config.down_buffer.write_position;
                    let read_position = &config.down_buffer.read_position;

                    // ensure all reads/writes to position data has already happened
                    fence(Ordering::SeqCst);
                    while self.rx_cursor.get() < self.rx_len.get()
                        && write_position.get() != read_position.get()
                    {
                        buffer[self.rx_cursor.get()] =
                            config.down_buffer[read_position.get() as usize].get();
                        // ensure output data ordered before updating read_position
                        fence(Ordering::SeqCst);
                        read_position.set((read_position.get() + 1) % config.down_buffer.length);
                        self.rx_cursor.set(self.rx_cursor.get() + 1);
                    }
                    // "flush" the final rx_cursor update
                    fence(Ordering::SeqCst);
                });
                if self.rx_cursor.get() == self.rx_len.get() {
                    client.received_buffer(buffer, self.rx_len.get(), Ok(()), uart::Error::None);
                } else {
                    let delay = self.alarm.ticks_from_ms(RX_MS_DELAY);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                    self.rx_client_buffer.put(Some(buffer))
                }
            });
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
        self.rx_client_buffer.put(Some(buffer));
        self.rx_len.set(len);
        self.rx_cursor.set(0);
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
