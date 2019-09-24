//! Segger RTT implementation.
//!
//! RTT is a protocol for sending debugging messages to a connected host. The
//! embedded platform configures a portion of memory in a special way, and then
//! the host uses a JTAG connection to read the messages out of the chip's
//! memory.
//!
//!	Receiving RTT Messages
//!	----------------------
//!
//!	With the jlink tools, reciving RTT messages is a two step process. First,
//!	open a JTAG connection with a command like:
//!
//!         $ JLinkExe -device nrf52 -if swd -speed 1000 -autoconnect 1
//!
//!	Then, use the `JLinkRTTClient` tool in a different terminal to print the
//!	messages:
//!
//!         $ JLinkRTTClient
//!
//! Notes
//! -----
//!
//! This capsule requires a timer, but the timer is only there to defer the
//! `transmit_complete` callback until the next scheduler loop. In the future,
//! if there is support for software interrupts or deferred calls in capsules,
//! this timer should be removed.
//!
//! Todo
//! ----
//!
//! - Implement receive functionality.
//!
//! Usage
//! -----
//!
//! ```
//! pub struct Platform {
//!     // Other fields omitted for clarity
//!     console: &'static capsules::console::Console<
//!         'static,
//!         capsules::segger_rtt::SeggerRtt<
//!             'static,
//!             capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!         >,
//!     >,
//! }
//! ```
//!
//! In `reset_handler()`:
//!
//! ```
//! let virtual_alarm_rtt = static_init!(
//!     capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!     capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
//! );
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
//!     capsules::console::Console<
//!         'static,
//!         capsules::segger_rtt::SeggerRtt<
//!             capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!         >,
//!     >,
//!     capsules::console::Console::new(
//!         rtt,
//!         0, // Baud rate is meaningless with RTT
//!         &mut capsules::console::WRITE_BUF,
//!         &mut capsules::console::READ_BUF,
//!         kernel::Grant::create()
//!     )
//! );
//! kernel::hil::uart::UART::set_client(rtt, console);
//! console.initialize();
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::time::Frequency;
use kernel::hil::uart;
use kernel::ReturnCode;

/// Buffer for transmitting to the host.
pub static mut UP_BUFFER: [u8; 1024] = [0; 1024];

/// Buffer for receiving messages from the host.
pub static mut DOWN_BUFFER: [u8; 32] = [0; 32];

/// This structure is defined by the segger RTT protocol. It must exist in
/// memory in exactly this form so that the segger JTAG tool can find it in the
/// chip's memory and read and write messages to the appropriate buffers.
#[repr(C)]
pub struct SeggerRttMemory {
    id: [u8; 16],
    number_up_buffers: u32,
    number_down_buffers: u32,
    up_buffer: SeggerRttBuffer,
    down_buffer: SeggerRttBuffer,
}

#[repr(C)]
pub struct SeggerRttBuffer {
    name: *const u8, // Pointer to the name of this channel. Must be a 4 byte thin pointer.
    buffer: *const u8, // Pointer to the buffer for this channel.
    length: u32,
    write_position: u32,
    read_position: u32,
    flags: u32,
}

impl SeggerRttMemory {
    pub fn new(
        up_buffer_name: &'a [u8],
        up_buffer: &'static mut [u8],
        down_buffer_name: &'static [u8],
        down_buffer: &'static mut [u8],
    ) -> SeggerRttMemory {
        SeggerRttMemory {
            // Must be "SEGGER RTT".
            id: *b"SEGGER RTT\0\0\0\0\0\0",
            number_up_buffers: 1,
            number_down_buffers: 1,
            up_buffer: SeggerRttBuffer {
                name: up_buffer_name.as_ptr(),
                buffer: up_buffer.as_ptr(),
                length: 1024,
                write_position: 0,
                read_position: 0,
                flags: 0,
            },
            down_buffer: SeggerRttBuffer {
                name: down_buffer_name.as_ptr(),
                buffer: down_buffer.as_ptr(),
                length: 32,
                write_position: 0,
                read_position: 0,
                flags: 0,
            },
        }
    }
}

pub struct SeggerRtt<'a, A: hil::time::Alarm<'a>> {
    alarm: &'a A, // Dummy alarm so we can get a callback.
    config: TakeCell<'a, SeggerRttMemory>,
    up_buffer: TakeCell<'static, [u8]>,
    _down_buffer: TakeCell<'static, [u8]>,
    client: OptionalCell<&'a uart::TransmitClient>,
    client_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
}

impl<'a, A: hil::time::Alarm<'a>> SeggerRtt<'a, A> {
    pub fn new(
        alarm: &'a A,
        config: &'a mut SeggerRttMemory,
        up_buffer: &'static mut [u8],
        down_buffer: &'static mut [u8],
    ) -> SeggerRtt<'a, A> {
        SeggerRtt {
            alarm: alarm,
            config: TakeCell::new(config),
            up_buffer: TakeCell::new(up_buffer),
            _down_buffer: TakeCell::new(down_buffer),
            client: OptionalCell::empty(),
            client_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
        }
    }
}

impl<'a, A: hil::time::Alarm<'a>> uart::Uart<'a> for SeggerRtt<'a, A> {}
impl<'a, A: hil::time::Alarm<'a>> uart::UartData<'a> for SeggerRtt<'a, A> {}

impl<'a, A: hil::time::Alarm<'a>> uart::Transmit<'a> for SeggerRtt<'a, A> {
    fn set_transmit_client(&self, client: &'a uart::TransmitClient) {
        self.client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.up_buffer.is_some() && self.config.is_some() {
            self.up_buffer.map(|buffer| {
                self.config.map(move |config| {
                    // Copy the incoming data into the buffer. Once we increment
                    // the `write_position` the RTT listener will go ahead and read
                    // the message from us.
                    let mut index = config.up_buffer.write_position as usize;
                    let buffer_len = config.up_buffer.length as usize;

                    for i in 0..tx_len {
                        buffer[(i + index) % buffer_len] = tx_data[i];
                    }

                    index = (index + tx_len) % buffer_len;
                    config.up_buffer.write_position = index as u32;
                    self.tx_len.set(tx_len);
                    // Save the client buffer so we can pass it back with the callback.
                    self.client_buffer.replace(tx_data);

                    // Start a short timer so that we get a callback and
                    // can issue the callback to the client.
                    let interval = (100 as u32) * <A::Frequency>::frequency() / 1000000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                })
            });
            (ReturnCode::SUCCESS, None)
        } else {
            (ReturnCode::EBUSY, Some(tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl<A: hil::time::Alarm<'a>> hil::time::AlarmClient for SeggerRtt<'a, A> {
    fn fired(&self) {
        self.client.map(|client| {
            self.client_buffer.take().map(|buffer| {
                client.transmitted_buffer(buffer, self.tx_len.get(), ReturnCode::SUCCESS);
            });
        });
    }
}

// Dummy implementation so this can act as the underlying UART for a
// virtualized UART MUX. -pal 1/10/19
impl<'a, A: hil::time::Alarm<'a>> uart::Configure for SeggerRtt<'a, A> {
    fn configure(&self, _parameters: uart::Parameters) -> ReturnCode {
        ReturnCode::FAIL
    }
}

// Dummy implementation so this can act as the underlying UART for a
// virtualized UART MUX.  -pal 1/10/19
impl<'a, A: hil::time::Alarm<'a>> uart::Receive<'a> for SeggerRtt<'a, A> {
    fn set_receive_client(&self, _client: &'a uart::ReceiveClient) {}
    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        _len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::FAIL, Some(buffer))
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}
