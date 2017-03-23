Kernel 802.15.4 Radio HIL
========================================

**TRD:**  <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Authors:** Philip Levis <br/>
**Draft-Created:** Feb 14, 2017<br/>
**Draft-Modified:** Mar 20, 2017<br/>
**Draft-Version:** 2<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL)
for an 802.15.4 radio in the Tock operating system kernel. It describes
the Rust traits and other definitions for this service as well as the
reasoning behind them. This document is in full compliance
with [TRD1].

1 Introduction
========================================

Wireless communication is an integral component of sensor networks and
the Internet of Things (IoT). 802.15.4 is low-power link layer that is
well suited to ad-hoc and mesh networks. It underlies numerous network
technologies, such as ZigBee, 6lowpan, and Thread, and there is a large
body of research on how to use it for extremely robust and low-power
networking. With a maximum frame size of 128 bytes, simple but effective
coding to reduce packet losses,  multiple addressing modes, AES-based
cryptograpy, and synchronous link-layer acknowledgments, 802.15.4 is
a flexible and efficient link layer for many applications and uses.

This document describes Tock's HIL for an 802.15.4 radio. The HIL is
in the kernel create, in model hil::radio. It provides four traits:

  * kernel::hil::radio::RadioControl: turn the radio on/off and configure it
  * kernel::hil::radio::Radio: send, receive and access packets
  * kernel::hil::radio::TxClient: handles callback when transmission completes
  * kernel::hil::radio::RxClient: handles callback when packet received
  * kernel::hil::radio::ConfigClient: handles callback when configuration
    changed

The rest of this document discusses each in turn.

2 Configuration constants and buffer management
========================================

To avoid extra buffers and memory copies, the radio stack requires that
callers provide it with memory buffers that are larger than the maximum
frame size it can send/receive. A caller provides a single, contiguous
buffer of memory. The frame itself is at an offset within his buffer,
and the data payload is at an offset from the beginnig of the frame.
The <a href="#impl">implementation section</a> gives a detailed example
of this layout for the RF233 radio.

Following this approach, The Radio HIL defines 4 constants:

  * kernel::hil::radio::HEADER_SIZE: the size of an 802.15.4 header,
  * kernel::hil::radio::MAX_PACKET_SIZE: the maximum frame size,
  * kernel::hil::radio::MAX_BUF_SIZE: the size buffer that must be
    provided to the radio, and
  * kernel::hil::radio::MIN_PACKET_SIZE: the smallest frame that can
    be received (typically HEADER_SIZE + 2 for an error-detecting CRC).

Note that MAX_BUF_SIZE can be larger (but not smaller) than MAX_PACKET_SIZE.
A radio must be given receive buffers that are MAX_BUF_SIZE in order to
ensure that it can receive maximum length packets.

3 RadioControl trait
========================================

The RadioControl trait provides functions to initialize an 802.15.4 radio,
turn it on/off and configure it.

3.1 Changing radio power state
-------------------------------

    fn initialize(&self,
                  spi_buf: &'static mut [u8],
                  reg_write: &'static mut [u8],
                  reg_read: &'static mut [u8])
                  -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;

    fn is_on(&self) -> bool;
    fn busy(&self) -> bool;
    fn set_power_client(&self, client: &'static PowerClient);

The `initialize` function takes three buffers, which are required for
the driver to be able to control the radio over an SPI bus. The first,
`spi_buf`, MUST have length MAX_BUF_SIZE. This buffer is required so that
the driver can interact over an SPI bus. An SPI bus usually requires both
a transmit and a receive buffer: software writes out the the TX buffer
(the MOSI line) while it reads into the RX buffer (MISO line). When
a caller tries to transmit a packet buffer, the radio needs an SPI receive
buffer to check the radio status. Similarly, when the stack receives
a packet into a buffer, it needs an SPI transmit buffer to send the command
to read from radio memory. The `spi_buf` buffer is purely internal, once
configured, it MUST never be visible outside of the stack.

The `reg_write` and `reg_read` buffers are needed to read and write
radio registers over the SPI bus. They are both 2 bytes long. These
buffers are purely internal and MUST never be visible outside the
stack.

The `reset` function resets the radio and configures its underlying
hardware resources (GPIO pins, buses, etc.). `reset` MUST be called
at before calling `start`.

The `start` function transitions the radio into a state in which it
can send and receive packets. It either returns FAIL because the
radio cannot be started or SUCCESS if it will be started. If the radio
is already started (or in the process), `start` MUST return FAIL. I.e.,
if software calls `start` twice, the second call would return FAIL.
Software can tell when the radio has completed initialization by
caling `started`.

The `stop` function returns the radio to a low-power state. The
function returns SUCCESS if the radio will transition to a
low-power state and FAIL if it will not. Software can tell when the
radio has turned off by calling `started`.

The `is_on` function returns whether the radio is in a powered-on
state. If the radio is on and can send/receive packets, it MUST return
true. If the radio cannot send/receive packets, it MUST return false.

The `busy` function returns whether the radio is currently busy.
It MUST return false if the radio is currently idle and can accept
reconfiguration or packet transmission requests. If it is busy and
cannot accept reconfiguration or packet transmission requests, it
MUST return true.

The `set_power_client` function allows a client to register a
callback for when the radio's power state changes.


3.2 Configuring the radio
-------------------------------

Re-configuring an 802.15.4 radio is an asynchronous operation.
Calling functions to change the radio's configuration does not
actually reconfigure it. Instead, those configuration changes
must be committed by calling `config_commit`. The radio issues a
callback when the reconfiguration completes. The object to receive
the callback is set by calling `set_config_client`. If `config_commit`
returns SUCCESS and there is a configuration client installed, the
radio MUST issue a `config_done` callback. `config_commit` MAY
return EOFF if the radio is off, or may return SUCCESS and hold the
configuration commit until the radio is turned on again.

    fn set_config_client(&self, client: &'static ConfigClient);
    fn config_commit(&self) -> ReturnCode;

A caller can configure the 16-bit short address, 64-bit full address,
PAN (personal area network) identifier, transmit power, and
channel. The PAN address and node address are both 16-bit values.
Channel is an integer in the range 11-26 (the 802.15.4 channel
numbers). `config_set_channel` MUST return EINVAL if passed a channel
not in the range 11-26 and SUCCESS otherwise.

    fn config_address(&self) -> u16;
    fn config_address_long(&self) -> [u8;8];
    fn config_pan(&self) -> u16;
    fn config_tx_power(&self) -> i8;
    fn config_channel(&self) -> u8;
    fn config_set_address(&self, addr: u16);
    fn config_set_address_long(&self, addr: [u8;8]);
    fn config_set_pan(&self, addr: u16);
    fn config_set_tx_power(&self, power: i8) -> ReturnCode;
    fn config_set_channel(&self, chan: u8) -> ReturnCode;

`config_set_tx_power` takes an signed integer, whose units are dBm.
If the specified value is greater than the maximum supported transmit
power or less than the minimum supported transmit power, it MUST
return EINVAL. Otherwise, it MUST set the transmit power to the
closest value that the radio supports. `config_tx_power` MUST return
the actual transmit power value in dBm. Therefore, it is possible that
the return value of `config_tx_power` returns a different (but close)
value than what it set in `config_set_tx_power`.

4 RadioData trait for sending and receiving packets
========================================

The RadioData trait implements the radio data path: it allows clients to
send and receive packets as well as accessors for packet fields.


    fn payload_offset(&self, long_src: bool, long_dest: bool) -> u8;
    fn header_size(&self, long_src: bool, long_dest: bool) -> u8;
    fn packet_header_size(&self, packet: &'static [u8]) -> u8;
    fn packet_get_src(&self, packet: &'static [u8]) -> u16;
    fn packet_get_dest(&self, packet: &'static [u8]) -> u16;
    fn packet_get_src_long(&self, packet: &'static [u8]) -> [u8;8]
    fn packet_get_dest_long(&self, packet: &'static [u8]) -> [u8;8];
    fn packet_get_pan(&self, packet: &'static [u8]) -> u16;
    fn packet_get_length(&self, packet: &'static [u8]) -> u8;
    fn packet_has_src_long(&self, packet: &'static [u8]) -> bool;
    fn packet_has_dest_long(&self, packet: &'static [u8]) -> bool;

The `packet_` functions MUST NOT be called on improperly formatted
802.15.4 packets (i.e., only on received packets). Otherwise the
return values are undefined.  `payload_offset` returns the offset in a
buffer at which the radio stack places the data payload. To send a
data payload, a client should fill in the payload starting at this
offset. For example, if `payload_offset` returns 11 and the caller
wants to send 20 bytes, it should fill in bytes 11-30 of the buffer
with the payload. `header_size` returns the size of a header based
on whether the source and destination addresses are long (64-bit)
or short (16-bit). `packet_header_size` returns the size of the
header on a particular correctly formatted packet (i.e., it looks
at the header to see if there are long or short addresses).

The data path has two callbacks: one for when a packet is received and
one for when a packet transmission completes.

    fn set_transmit_client(&self, client: &'static TxClient);
    fn set_receive_client(&self, client: &'static RxClient,
                          receive_buffer: &'static mut [u8]);
    fn set_receive_buffer(&self, receive_buffer: &'static mut [u8]);

Registering for a receive callback requires also providing a packet
buffer to receive packets into. The receive callback MUST pass this
buffer back. The callback handler MUST install a new receive buffer
with a call to `set_receive_buffer`. This buffer MAY be the same
buffer it received or a different one.

Clients transmit packets by calling `transmit` or `transmit_long`.

    fn transmit(&self,
                dest: u16,
                tx_data: &'static mut [u8],
                tx_len: u8,
                source_long: bool) -> ReturnCode;

    fn transmit_long(&self,
                dest: [u8;8],
                tx_data: &'static mut [u8],
                tx_len: u8,
                source_long: bool) -> ReturnCode;

The packet sent on the air by a call to `transmit` MUST be formatted
to have a 16-bit short destination address equal to the `dest`
argument. A packet sent on the air by a call to `transmit_long` MUST
be formatted to have a 64-bit destination address equal to the `dest`
argument.

The `source_long` parameter denotes the length of the source address in
the packet. If `source_long` is false, the implementation MUST include
a 16-bit short source address in the packet. If `source_long` is true,
the implementation MUST include a 64-bit full source address in the
packet. The addresses MUST be consistent with the values written and
read with `config_set_address`, `config_set_address_long`,
`config_address`, and `config_address_long`.

The passed buffer `tx_data` MUST be MAX_BUF_LEN in size.  `tx_len` is
the length of the payload. If `transmit` returns SUCCESS, then the
driver MUST issue a transmission completion callback. If `transmit`
returns any value except SUCCESS, it MUST NOT accept the packet for
transmission and MUST NOT issue a transmission completion callback. If
`tx_len` is too long, `transmit` MUST return ESIZE. If the radio is
off, `transmit` MUST return EOFF.  If the stack is temporarilt unable
to send a packet (e.g., already has a transmission pending), then
`transmit` MUST return EBUSY. If the stack accepts a packet for
transmission (returns SUCCESS), it MUST return EBUSY until it issues a
transmission completion callback.

5 TxClient, RxClient, ConfigClient, and PowerClient traits
========================================

An 802.15.4 radio provides four callbacks: packet transmission
completion, packet reception, when a change to the radio's
configuration has completed, and when the power state of the
radio has changed.

    pub trait TxClient {
        fn send_done(&self, buf: &'static mut [u8], acked: bool, result: ReturnCode);
    }

The `buf` paramater of `send_done` MUST pass back the same buffer that
was passed to `transmit`. `acked` specifies whether the sender
received a link-layer acknowledgement (indicating the packet was
successfully received). `result` indicates whether or not the packet
was transmitted successfully; it can take on any of the valid return
values for `transmit` or FAIL to indicate other reasons for failure.

The `receive` callback is called whenever the radio receives a packet
destined to the node's address (including broadcast address) and PAN
id that passes a CRC check. If a packet is not destined to the node or
does not pass a CRC check then `receive` MUST NOT be called. `buf` is
the buffer containing the received packet. It MUST be the same buffer
that was passed with either installing the receive handler or calling
`set_receive_buffer`. The buffer is consumed through the callback: the
radio stack MUST NOT maintain a reference to the buffer. A client that
wants to receive another packet MUST call `set_receive_buffer`.

    pub trait RxClient {
        fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
    }

The `config_done` callback indicates that a radio reconfiguration has
been committed to hardware. If the configuration has been successfully
committed, `result` MUST be SUCCESS. It may otherwise take on any
value that is a valid return value of `config_commit` or FAIL to
indicate another failure.

    pub trait ConfigClient {
        fn config_done(&self, result: ReturnCode);
    }

The `changed` callback indicates that the power state of the radio
has changed. The `on` parameter states whether it is now on or off.
If a call to `stop` using the RadioConfig interface returns SUCCESS,
the radio MUST issue a `changed` callback when the radio is powered
off, passing `false` as the value of the `on` parameter. If a
call to `start` using the RadioConfig interface returns SUCCESS,
the radio MUST issue a `changed` callback when the radio is powered
on, passing `true` as the value of the `on` parameter.

    pub trait PowerClient {
        fn changed(&self, on: bool);
    }

The return value of `is_on` MUST be consistent with the state as
exposed through the `changed` callback.  If the `changed` callback has
indicated that the radio is on, then `is_on` MUST return true a later
callback signals the radio is off. Similarly, if the `changed` callback
has indicated that the radio is off, then `is_on` MUST return false
until a later callback signals the radio is on.

6 RadioCrypto trait
========================================

The RadioCrypto trait is for configuring and enabling/disabling
different security settings.

7 Example Implementation: RF233
========================================

An implementation of the radio HIL for the Atmel RF233 radio can be
found in capsules::rf233. This implementation interacts with an RF233
radio over an SPI bus. It supports 16-bit addresses, intra-PAN
communication, and synchronous link-layer acknowledgments. It has two
files: `rf233.rs` and `rf233_const.rs`. The latter has constants such
as register identifiers, command formats, and register flags.

The RF233 has 6 major operations of the SPI bus: read a register,
write a register, read an 802.15.4 frame, write an 802.15.4 frame,
read frame SRAM and write frame SRAM. The distinction between frame and
SRAM access is that frame access always starts at index 0, while
SRAM access has random access (a frame operation is equivalent to an
SRAM operation with address 0). The implementation only uses register
and frame operations. The details of these operations can be found
in Section 6.3 of the RF233 datasheet [RF233].

The implementation has 6 high-level states:

  * off,
  * initializing the radio,
  * turning on the radio to receive,
  * waiting to receive packets (default idle state),
  * receiving a packet,
  * transmitting a packet, and
  * committing a configuration change.

All of these states, except off, have multiple substates.  They reach
represent a (mostly) linear series of state transitions. If a client
requests an operation (e.g., transmit a packet, reconfigure) while the
stack is in the waiting state, it starts the operation immediately. If
it is in the midst of receiving a packet, it marks the operation as
pending and completes it when it falls back to the waiting state. If
there is both a packet transmission and a reconfiguration pending, it
prioritizes the transmission first.

The RF233 provides an interrupt line to the processor, to denote some
state changes.  The radio has multiple interrupts, which are are
multiplexed onto a single interrupt line. Software is responsible for
reading an interrupt status register on the radio (a register read
operation) to determine what interrupts are pending. Since a register
read requires an SPI operation, it can be significantly delayed. For
example, if the stack is the midst of writing out a packet to the
radio's frame buffer, it will complete the SPI operation before
issuing a register read. In cases when transmissions are interrupted
by packet reception, the stack simply marks the packet as pending and
waits for the reception to complete, then retries the transmission.

8 Authors' Address
========================================

    Philip Levis
    409 Gates Hall
    Stanford University
    Stanford, CA 94305
    phone - +1 650 725 9046
    email - pal@cs.stanford.edu

9. Citations
========================================

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"

[RF233]: http://www.atmel.com/images/Atmel-8351-MCU_Wireless-AT86RF233_Datasheet.pdf "AT86RF233: Low Power, 2.4GHz Transceiver for ZigBee, RF4CE, IEEE 802.15.4, 6LoWPAN, and ISM Applications"
