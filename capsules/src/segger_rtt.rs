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
use core::marker::PhantomData;
use kernel::common::cells::{OptionalCell, TakeCell, VolatileCell};
use kernel::hil;
use kernel::hil::uart;
use kernel::ReturnCode;

/// Suggested length for the up buffer to pass to the Segger RTT capsule.
pub const DEFAULT_UP_BUFFER_LENGTH: usize = 1024;

/// Suggested length for the down buffer to pass to the Segger RTT capsule.
pub const DEFAULT_DOWN_BUFFER_LENGTH: usize = 32;

/// This structure is defined by the segger RTT protocol. It must exist in
/// memory in exactly this form so that the segger JTAG tool can find it in the
/// chip's memory and read and write messages to the appropriate buffers.
#[repr(C)]
pub struct SeggerRttMemory<'a> {
    id: VolatileCell<[u8; 16]>,
    number_up_buffers: VolatileCell<u32>,
    number_down_buffers: VolatileCell<u32>,
    up_buffer: SeggerRttBuffer<'a>,
    down_buffer: SeggerRttBuffer<'a>,
}

#[repr(C)]
pub struct SeggerRttBuffer<'a> {
    name: VolatileCell<*const u8>, // Pointer to the name of this channel. Must be a 4 byte thin pointer.
    // These fields are marked as `pub` to allow access in the panic handler.
    pub buffer: VolatileCell<*const u8>, // Pointer to the buffer for this channel.
    pub length: VolatileCell<u32>,
    pub write_position: VolatileCell<u32>,
    read_position: VolatileCell<u32>,
    flags: VolatileCell<u32>,
    _lifetime: PhantomData<&'a [u8]>,
}

impl SeggerRttMemory<'a> {
    pub fn new_raw(
        up_buffer_name: &'a [u8],
        up_buffer_ptr: *const u8,
        up_buffer_len: usize,
        down_buffer_name: &'a [u8],
        down_buffer_ptr: *const u8,
        down_buffer_len: usize,
    ) -> SeggerRttMemory<'a> {
        SeggerRttMemory {
            // This field is a magic value that must be set to "SEGGER RTT" for the debugger to
            // recognize it when scanning the memory.
            //
            // In principle, there could be a risk that the value is duplicated elsewhere in
            // memory, therefore confusing the debugger. However in practice this hasn't caused any
            // known problem so far. If needed, this ID could be scrambled here, with the real magic
            // value being written only when this object is fully initialized.
            id: VolatileCell::new(*b"SEGGER RTT\0\0\0\0\0\0"),
            number_up_buffers: VolatileCell::new(1),
            number_down_buffers: VolatileCell::new(1),
            up_buffer: SeggerRttBuffer {
                name: VolatileCell::new(up_buffer_name.as_ptr()),
                buffer: VolatileCell::new(up_buffer_ptr),
                length: VolatileCell::new(up_buffer_len as u32),
                write_position: VolatileCell::new(0),
                read_position: VolatileCell::new(0),
                flags: VolatileCell::new(0),
                _lifetime: PhantomData,
            },
            down_buffer: SeggerRttBuffer {
                name: VolatileCell::new(down_buffer_name.as_ptr()),
                buffer: VolatileCell::new(down_buffer_ptr),
                length: VolatileCell::new(down_buffer_len as u32),
                write_position: VolatileCell::new(0),
                read_position: VolatileCell::new(0),
                flags: VolatileCell::new(0),
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
}

pub struct SeggerRtt<'a, A: hil::time::Alarm<'a>> {
    alarm: &'a A, // Dummy alarm so we can get a callback.
    config: TakeCell<'a, SeggerRttMemory<'a>>,
    up_buffer: TakeCell<'a, [u8]>,
    _down_buffer: TakeCell<'a, [u8]>,
    client: OptionalCell<&'a dyn uart::TransmitClient>,
    client_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
}

impl<'a, A: hil::time::Alarm<'a>> SeggerRtt<'a, A> {
    pub fn new(
        alarm: &'a A,
        config: &'a mut SeggerRttMemory<'a>,
        up_buffer: &'a mut [u8],
        down_buffer: &'a mut [u8],
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
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
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
                    let mut index = config.up_buffer.write_position.get() as usize;
                    let buffer_len = config.up_buffer.length.get() as usize;

                    for i in 0..tx_len {
                        buffer[(i + index) % buffer_len] = tx_data[i];
                    }

                    index = (index + tx_len) % buffer_len;
                    config.up_buffer.write_position.set(index as u32);
                    self.tx_len.set(tx_len);
                    // Save the client buffer so we can pass it back with the callback.
                    self.client_buffer.replace(tx_data);

                    // Start a short timer so that we get a callback and
                    // can issue the callback to the client.
                    let delay = A::ticks_from_us(100);
                    self.alarm.set_alarm(self.alarm.now(), delay);
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
    fn alarm(&self) {
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
    fn set_receive_client(&self, _client: &'a dyn uart::ReceiveClient) {}

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
