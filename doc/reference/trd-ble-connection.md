Kernel BLE Connection HIL
=========================

**TRD:**  <br/>
**Working Group:** Network<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Authors:** Amit Levy <br/>
**Draft-Created:** Nov 20, 2026<br/>
**Draft-Modified:** Nov 20, 2026<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org</br>

Abstract
---------

This document describes the hardware independent layer interface (HIL)
for Bluetooth Low Energy (BLE) connections in the Tock operating
system kernel, for the case where the link layer runs in software on a
directly programmable 2.4 GHz radio. It describes the Rust traits and
the reasoning behind them, explains why this is only one of two
plausible controller interfaces for BLE, and records a
radio-arbitration constraint that applies to dual-radio-stack
chips. This document is in full compliance with [TRD1].

This is a draft, and the design has only been validated on one chip,
the nRF52840. Section 6 explains why that matters and what would
change.

1 Introduction
===============

Tock has supported BLE *advertising* since 2017, through
`kernel::hil::ble_advertising` and the `ble_advertising_driver`
capsule. It advertises or listens for broadcasts, but does not
establish a link.

With BLE *connections*, once a peripheral accepts a `CONNECT_IND`
exchange, it owes the central a packet at every connection event on a
channel determined by a central-defined hopping sequence, within a
window whose width is set by the two devices' clock
accuracy. Furthermore, the peripheral must reply to each packet it
receives exactly 150 us later. Some missed packets are permitted, but
too many missed packets drops the link.

This document specifies the kernel interface that makes that possible
on a chip whose radio is directly programmable, and describes portable
layers that sit above it.

1.1 Scope
-----------

In scope: the peripheral role with one connection at a time, the LE 1M
PHY, advertising, and the unencrypted ACL data path up through GATT.

Out of scope: the central role, multiple simultaneous
connections, the Security Manager, L2CAP connection-oriented channels,
isochronous channels (LE Audio), and the extended advertising features of BLE
5.x. Section 7 says what reserving room for these implies.

2 The BLE stack and what is timing-critical
===========================================

```
 Host    GATT   (services / characteristics semantics)   -- portable
         ATT    (attribute PDUs: Read/Write/Notify...)
         SMP / L2CAP-signalling
         L2CAP  (multiplex by CID)
 ---------------------------------------------------------------------
 Ctrlr   Link Layer (connections, acknowledgement, hopping, control)  -- TIMING-CRITICAL
         Physical Layer (radio)
```

Only the link layer and below are timing-critical. The inter-frame space between
a received packet and its reply is fixed at T_IFS = 150 us (Vol 6, Part B
4.1.1). No general-purpose operating system---Tock included---can schedule a
software response that fast and that reliably.

The consequence shapes this entire interface: **the turnaround must be performed
by hardware, so the kernel interface must let a chip driver own it.** On the
nRF52 this is a PPI route from a timer compare to a radio task, plus radio
SHORTS, plus the `TIFS` register. The CPU's only job during a connection event is
to fix up two bits of the outgoing packet header during the transmit ramp-up.

Everything above the link layer is latency-tolerant. A GATT read may be answered
several connection events later with no ill effect. Those layers therefore live
in portable capsules and are not described by this HIL at all.

3 Two controller interfaces, not one
========================================

Tock should eventually support BLE on more than one chip, and in
practice, chips do not agree on where the hardware/software boundary
lies.

**Category 1: raw radio.** The chip exposes a programmable 2.4 GHz
radio and the link layer runs in software on the MCU. Nordic's nRF5x
family is the main example. This document focuses only on this
category.

**Category 2: HCI controller.** The link layer already runs on
dedicated hardware---typically a coprocessor core or a separate
chip---and the host talks to it over the Host Controller Interface, a
Bluetooth SIG standard. Examples include STM32WB, ESP32, NXP KW45,
Silicon Labs EFR32, and Infineon AIROC. Tock already has one such
driver: `chips/apollo3/src/ble.rs` shovels HCI commands to an on-die
BLE core over a FIFO. Notably, its `transmit_advertisement()`
implementation ignores the channel argument, because on that chip the BLE
controller selects channels itself.

The vast majority of chips available are in category 2. For
comparison, Zephyr only runs its own controller on Nordic chips, and
implements BLE on other chip families through an HCI driver.

As a result, the traits in Section 4 are category 1 interfaces and are
not intended to be a portable interface boundary for Tock more
generally.

Portable layers above introduced in Section 5---L2CAP, ATT, and
GATT---are expected to survive the introduction of a category 2
boundary unchanged.

4 The connection HIL
=====================

The traits live in `kernel::hil::ble_connection`, separate from
`kernel::hil::ble_advertising`, because they describe a different
capability that a chip may implement independently.

4.1 ConnectionParams
---------------------

`ConnectionParams` carries the contents of the `CONNECT_IND` LLData
field (Vol 6, Part B 2.3.3.1): the access address, CRC initialisation
value, channel map, hop increment, connection interval, slave latency,
supervision timeout, and transmit window size and offset.

While the Bluetooth spec expresses durations in units of 1.25 ms or 10
ms, all durations in `ConnectionParams` are converted to microseconds
or milliseconds directly.

4.2 BleConnectionDriver
-----------------------

```rust
pub trait BleConnectionDriver<'a>: kernel::hil::time::Time {
    fn connection_configure(&self, params: &ConnectionParams) -> Result<(), ErrorCode>;

    fn connection_event_start(
        &self,
        channel: RadioChannel,
        buffers: TxBuffers,
        open_time: Self::Ticks,
    ) -> Result<(), (ErrorCode, TxBuffers)>;

    fn set_connection_event_client(&self, client: &'a dyn ConnectionEventClient<Self::Ticks>);
}
```

`connection_configure()` applies the per-connection access address and CRC
initialisation value and prepares the radio's inter-frame timing. It is a control
operation with no completion callback.

`connection_event_start()` is split-phase: `Ok(())` means
`ConnectionEventClient::connection_event_done()` MUST be called, and
an error hands the buffers back. It arms the radio to begin receiving
on `channel` at `open_time` and hands over the two candidate PDUs, one
of which it will transmit in response.

**`Time` supertrait:** A connection event is scheduled at an absolute
time, and the client needs to read the same clock to compute
it. Rather than expose a bespoke `now()`, the trait requires
`kernel::hil::time::Time`, so the clock's width and frequency are part
of the type and the client can use the standard `Ticks` and
`ConvertTicks` helpers. An implementation MUST offer at least
microsecond resolution. The nRF52 implementation uses `Freq1MHz` and
`Ticks32`.

**`TxBuffers`:** The central acknowledges transmission from the
previous connection event at the beginning of each new connection
event (Vol 6, Part B 4.5.9). So, when starting a new connection event,
a peripheral has at most one payload outstanding that has not been
acknowledged yet. Until that payload is acknowledged it MUST be
retransmitted unchanged. Once it is acknowledged, sending it again
under the advanced sequence number would reach the central as a new
and duplicate packet.

Acknowledgements arrive one inter-frame space---150 us---before the
peripheral must reply one way or the other. Since this is too tight to
consult a client in between reception and transmission, the client
provides both candidates before the event starts:

```rust
pub struct TxBuffers {
    /// Transmitted in the previous connection event, and retransmitted until
    /// the central acknowledges it.
    pub outstanding: &'static mut [u8],
    /// Transmitted as soon as `outstanding` is acknowledged.
    pub next: &'static mut [u8],
}
```

The driver sends `next` if the packet it has just received
acknowledges the outstanding payload, and `outstanding`
otherwise. Each buffer MUST hold a complete link-layer PDU; a client
with nothing to send places an empty PDU there.

4.3 ConnectionEventClient
--------------------------

```rust
pub trait ConnectionEventClient<T: Ticks> {
    fn connection_event_done(
        &self,
        rx: &[u8],
        buffers: TxBuffers,
        result: Result<(), ErrorCode>,
        anchor: T,
        tx_acked: bool,
    );
}
```

`connection_event_done` MUST be called once per connection event,
after both the receive and the transmit have completed.

The received packet and the transmit buffers are handled
differently. `buffers` belong to the client: they were handed to the
driver and are handed back, per the HIL convention that a driver never
retains a `&'static mut` buffer across a completed operation. `rx` is
only borrowed for the duration of the call, because the driver never
gives it up---the radio receives into the same buffer on every event.

`anchor` is the time at which the central's access address was
detected. `anchor` SHOULD be captured in radio hardware rather than by
a software timestamp, which carries enough jitter to accumulate into a
dropped link. The client SHOULD use it to re-anchor its schedule.

`tx_acked` reports whether the central acknowledged the previous
transmission. It tells the client which buffer was transmitted, and
so when to advance: on `true` the driver sent `next`, which becomes
the new `outstanding`, freeing the buffer it displaces to take
whatever follows.

4.4 ConnectionSetupClient and DisconnectClient
-----------------------------------------------

`ConnectionSetupClient::connect_ind_received()` hands parsed connection
parameters from whichever component received the `CONNECT_IND` to whichever
component will drive the connection. `DisconnectClient::connection_lost()`
reports the reverse transition, so an advertiser can resume.

These two traits exist to keep the advertising capsule and the
connection capsule from depending on each other directly.

5 Portable layers above the controller
========================================

The link-layer connection manager, L2CAP demultiplexer, ATT responder,
and GATT server are capsules. They touch no chip registers and are not
described by this HIL.

One interface among them is worth recording here because it determines whether
the work extends to userspace. The GATT server does not own its attributes; it is
given a table:

```rust
pub struct Service<'a> {
    pub uuid: u16,
    pub characteristics: &'a [Characteristic<'a>],
}

pub struct Characteristic<'a> {
    pub uuid: u16,
    pub properties: u8,
    pub value: &'a dyn AttributeProvider,
}

pub trait AttributeProvider {
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize, AttError>;
    fn write(&self, offset: usize, value: &[u8]) -> Result<(), AttError>;
}
```

Handles are assigned by position rather than written down: a service takes one
handle for its declaration and each of its characteristics takes two. A board
therefore cannot produce a table whose handles disagree with its contents.

Today a board supplies this table. The reason for the indirection through
`AttributeProvider` is what comes next: a capsule that multiplexes GATT across
processes contributes a service per process and implements the trait for the
attributes that process owns. The ATT and L2CAP layers do not change when that
happens.

Note that dispatch here is synchronous. This is a deliberate constraint on the
eventual process-facing design: a process publishes a characteristic's value into
a buffer the capsule can read, rather than the capsule making a round trip to the
process on each read. ATT responses are queued and transmitted on a later
connection event, so an asynchronous provider remains possible if a use case
demands it.

6 Radio arbitration on nRF52
=============================

This section records a constraint that is not specific to connections but that
connections make unavoidable.

The nRF52840 has **one** 2.4 GHz radio able to support multiple
protocols. Tock has two drivers for it: `chips/nrf52/src/ble_radio.rs`
and `chips/nrf52/src/ieee802154_radio.rs`. These both wrap the same
peripheral at `0x40001000`. There is also only one `RADIO` interrupt.

The current arbitration is weak. Both drivers implement `is_enabled()`
by reading the shared `MODE` register and asking whether it names
their own protocol. Interrupt dispatch
(`chips/nrf52840/src/interrupt_service.rs`) matches on the pair, and
its "both enabled" arm is technically unreachable because `MODE` can
hold exactly one value. This means that whichever driver wrote `MODE`
last takes the radio, and the other could stop receiving its
interrupts mid-operation.

Advertising tolerates this, sometimes serendipitously. An advertiser
occupies the radio for a few milliseconds every couple of hundred
milliseconds and disables it in between, so two stacks interleave by
luck. In connection-oriented BLE, this is unlikely to succeed. The
peripheral must be on the air at every anchor point or the supervision
timer expires and the link drops.

Genuine coexistence requires a radio arbiter that grants timeslots to
protocol stacks with priorities and preemption. Nordic's Multiprotocol
Service Layer plays this role, and is the only way Thread and BLE
coexist in other systems on the nRF52s.

7 Future work
==============

**An HCI interface.** Section 3 argues this is where breadth across
chips comes from.

**Radio arbitration.** See Section 6.

**GATT virtualization across processes.** BLE provides exactly one ATT
channel per connection (fixed CID 0x0004), so processes cannot be
given separate channels. Multiplexing must happen in the GATT handle
space: one device identity, one connection, an attribute table
composed from per-process contributions, with operations routed to the
owning process.

**Isochronous channels.** LE Audio does not travel over L2CAP. It is a
sibling of the ACL data path at the controller boundary with its own
adaptation layer and real-time semantics. It does not disturb the
design above, but a complete subsystem architecture should reserve a
place for it beside L2CAP.

8 References
=============

- [TRD1]: Tock Reference Documents, `doc/reference/trd1-trds.md`
- Bluetooth Core Specification, Vol 6 Part B: Link Layer Specification.
  2.3.3.1 (CONNECT_IND), 4.5 (connection setup and timing), 4.5.8 (channel
  selection algorithm #1), 4.5.9 (acknowledgement and flow control).
- Bluetooth Core Specification, Vol 3 Part A (L2CAP), Part F (ATT), Part G (GATT).
- Tock issue #4400, "BLE Support Planning".

9 Authors' Address
===================
```
Amit Levy
amit@amitlevy.com
```
