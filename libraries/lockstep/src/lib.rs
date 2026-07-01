// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Arch-agnostic dual-core software lockstep library.
//!
//! Provides the shared types and primitives used by both the `qemu_rv32_virt`
//! (RISC-V BiChannel + CLINT MSIP) and `rp2350` (SIO inter-core FIFO + shared
//! SRAM) lockstep implementations. Each board supplies a concrete [`Transport`]
//! impl; the logic here is independent of any particular transport mechanism.
//!
//! # Architecture
//!
//! ```text
//!  Leader core (0)          Shadow core (1)
//!  ┌────────────────┐       ┌────────────────┐
//!  │ LockstepDriver │─────▶ │ LockstepDriver │  Layer-2 syscall gate
//!  │  (SyscallDriver│       │  (SyscallDriver│
//!  │   wrapper)     │       │   wrapper)     │
//!  └────────────────┘       └────────────────┘
//!  ┌────────────────┐       ┌────────────────┐
//!  │  LockstepUart  │──────▶│  LockstepUart  │  Layer-1 UART replay
//!  │  (UART wrapper)│       │  (UART wrapper)│
//!  └────────────────┘       └────────────────┘
//!          │   Transport::try_push / try_pop    │
//!          └────────────────────────────────────┘
//! ```
//!
//! # Two-layer model
//!
//! - **Layer 1** (kernel↔HIL): the leader reads real hardware inputs (UART RX,
//!   RNG) and forwards raw bytes to the shadow via [`Transport::bulk_write`];
//!   the shadow replays them into replica HIL drivers. Events are notified via
//!   [`SyncEntry::UartRxReady`] / [`SyncEntry::UartTxDone`] control words.
//! - **Layer 2** (syscall↔userspace): [`LockstepDriver`] intercepts every
//!   `Command` syscall, exchanges a [`SyncEntry::SyscallDesc`] across cores,
//!   and gates the leader on the shadow's confirmation before emitting output.

#![no_std]

use kernel::collections::spsc_channel::SpscChannel;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;

// ---------------------------------------------------------------------------
// BulkTag — labels for named shared-SRAM slots
// ---------------------------------------------------------------------------

/// Labels for [`Transport::bulk_write`] / [`Transport::bulk_read`] calls.
///
/// Each tag identifies a specific shared-SRAM region so the transport impl
/// can route data to the right buffer (on the RP2350 these are explicit
/// sub-regions of the shared SRAM partition; on QEMU they are `.bss` statics
/// accessed via PC-relative addressing).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BulkTag {
    /// UART RX bytes forwarded from the leader to the shadow.
    UartRx,
}

// ---------------------------------------------------------------------------
// SyncEntry — inter-core control messages
// ---------------------------------------------------------------------------

/// Payload of a Layer-2 syscall descriptor exchanged between cores.
///
/// Extracted from [`SyncEntry::SyscallDesc`] so it can be stored in the
/// shadow's pending queue ([`PENDING_SYSCALLS_SHADOW`]) without carrying the
/// full [`SyncEntry`] discriminant and other variants' data.
///
/// `payload_fp` is an FNV-1a fingerprint of the app's RO-allow buffer at the
/// configured slot (0 if no payload slot is wired up for this driver).
#[derive(Clone, Copy)]
pub struct SyscallDesc {
    pub driver_num: u32,
    pub sub: u8,
    pub arg0: u32,
    pub arg1: u32,
    pub payload_fp: u32,
}

/// Entry type for the inter-core lockstep channel.
///
/// Mirrors the RP2350 SIO inter-core FIFO design: one channel pair (one
/// direction each way) carries *all* inter-core communication distinguished
/// by tag, rather than a separate channel per purpose.
///
/// The leader pushes `Sync` once per kernel-loop iteration (and once at init
/// as a pure handshake), and pushes event variants from its trap handler.
/// The shadow drains the channel in a loop, dispatching each event
/// immediately, until it pops a `Sync`.
#[derive(Clone, Copy)]
pub enum SyncEntry {
    /// Per-iteration lockstep barrier and the one-time init handshake.
    ///
    /// `fingerprint` is the `KernelActivity` fingerprint for this iteration;
    /// the shadow echoes back its own so the leader can compare.
    Sync { fingerprint: u32 },

    /// Leader finished a UART RX completion; `len` bytes are waiting in the
    /// [`BulkTag::UartRx`] shared-SRAM slot for the shadow to replay.
    UartRxReady { len: u8 },

    /// Leader finished transmitting whatever the shadow UART buffer had queued.
    UartTxDone,

    /// Layer-2 syscall descriptor: the leader pushes this before dispatching
    /// each intercepted `Command` syscall; the shadow stores it during its
    /// Phase-1 drain and pops it in Phase 2 for comparison.
    /// See [`LockstepDriver`] and [`store_pending_syscall`].
    SyscallDesc(SyscallDesc),

    /// Layer-2 upcall descriptor: both cores exchange this at each intercepted
    /// upcall boundary to verify argument equivalence before delivering the
    /// upcall to userspace.
    ///
    /// `driver_num` / `subscribe_num` identify the upcall; `r0`–`r2` carry the
    /// masked argument values to compare (unmasked fields are zeroed).
    ///
    /// Note: at ~20 bytes this is wider than the 32-bit RP2350 SIO FIFO entry
    /// the channel models. Acceptable on a software channel (QEMU); a faithful
    /// RP2350 port requires word-packing or a shared-SRAM payload slot.
    UpcallDesc {
        driver_num: u32,
        subscribe_num: u8,
        r0: u32,
        r1: u32,
        r2: u32,
    },
}

// ---------------------------------------------------------------------------
// Transport — arch-specific inter-core communication
// ---------------------------------------------------------------------------

/// Arch-specific inter-core transport abstraction.
///
/// Implementors supply the platform-specific mechanics for:
/// - identifying which core we are (`core_id`),
/// - sending/receiving [`SyncEntry`] control words (`try_push` / `try_pop`),
/// - waking the peer core (`kick_peer`),
/// - transferring bulk data (UART RX bytes, etc.) via shared memory
///   (`bulk_write` / `bulk_read`),
/// - any chip-specific housekeeping done each spin iteration (`on_spin`).
///
/// Two implementations are planned:
/// - **`QemuTransport`** (`chips/qemu_rv32_virt_chip`): software
///   `BiChannel<4, SyncEntry>` in shared `.bss`, CLINT MSIP doorbells, CLINT
///   `mtime` for timeouts, `csrr mhartid` for core id.
/// - **`Rp2350Transport`** (`chips/rp2350`): native SIO inter-core FIFO
///   (32-bit × 4-deep), shared-SRAM bulk regions, TIMER0 `timerawl`,
///   `sio.get_processor()` for core id; FIFO write auto-kicks the peer so
///   `kick_peer` is a no-op.
pub trait Transport {
    /// Return the id of the *calling* core (0 = leader, 1 = shadow).
    fn core_id(&self) -> u8;

    /// Return a monotonically increasing tick count (µs-ish resolution).
    ///
    /// Used only for bounded deadline checks via `wrapping_sub`; the period
    /// must be far longer than any expected `SYNC_TIMEOUT_TICKS` or
    /// `DRAIN_TIMEOUT_TICKS`.
    fn now_ticks(&self) -> u32;

    /// Non-blocking push of a control word to the peer.
    ///
    /// Returns `true` if the word was queued, `false` if the channel was full.
    /// The caller retries on `false`. Does *not* kick the peer — call
    /// [`kick_peer`] after a successful push when a wakeup is needed.
    ///
    /// On the RP2350, the SIO FIFO write IS the push AND the kick (hardware
    /// auto-raises `SIO_IRQ_FIFO` on the peer), so `try_push` implicitly kicks
    /// and `kick_peer` is a no-op on that platform.
    fn try_push(&self, e: SyncEntry) -> bool;

    /// Non-blocking receive of a control word from the peer.
    ///
    /// Returns `Some` if an entry was available, `None` if the channel was
    /// empty. The caller polls in a loop.
    fn try_pop(&self) -> Option<SyncEntry>;

    /// Kick the peer core to process a just-pushed control word.
    ///
    /// On QEMU: writes the CLINT MSIP register for the peer hart. On RP2350:
    /// no-op (the SIO FIFO write already raised `SIO_IRQ_FIFO`).
    fn kick_peer(&self);

    /// Write `bytes` into the named shared-SRAM bulk slot.
    ///
    /// The implementation must publish the data with a store-release fence
    /// (or equivalent) so that a subsequent [`try_push`] notification is
    /// guaranteed to be observed *after* the data.
    fn bulk_write(&self, tag: BulkTag, bytes: &[u8]);

    /// Read up to `out.len()` bytes from the named shared-SRAM bulk slot.
    ///
    /// Returns the number of bytes actually copied. The implementation must
    /// use a load-acquire fence (or equivalent) to observe data written by a
    /// [`try_push`]-signalled [`bulk_write`].
    fn bulk_read(&self, tag: BulkTag, out: &mut [u8]) -> usize;

    /// Optional hook called each spin iteration inside [`lockstep_barrier`]
    /// and [`LockstepDriver::command`]'s gate loop.
    ///
    /// Default is a no-op. `QemuTransport` uses this to call
    /// `clear_irq_active()` so the watchdog does not misidentify a long gate
    /// wait as a hung interrupt handler.
    #[inline(always)]
    fn on_spin(&self) {}

    /// Timeout for the Sync barrier spin-wait, in [`now_ticks`] units.
    ///
    /// Must be large enough to survive the worst-case asymmetric kernel-work
    /// burst between two consecutive barriers on the leader side.
    const SYNC_TIMEOUT_TICKS: u32;

    /// Timeout for the shadow's Phase-1 drain loop, in [`now_ticks`] units.
    ///
    /// After receiving an L1 event (UartTxDone / RngReady / UartRxReady) the
    /// shadow starts this timer. A matching `Sync` from the leader MUST follow
    /// before it expires; if not, the leader has diverged.
    const DRAIN_TIMEOUT_TICKS: u32;
}

// ---------------------------------------------------------------------------
// lockstep_barrier — symmetric rendezvous primitive
// ---------------------------------------------------------------------------

/// Exchange a synchronization descriptor with the peer and return its reply.
///
/// Both cores run identical code; only the [`Transport`] impl selects the
/// correct channel direction. Each iteration:
///
/// 1. Tries a non-blocking push of `mine` (retries until the channel has space).
///    Calls [`Transport::kick_peer`] once after the first successful push.
/// 2. Tries a non-blocking pop; if an entry arrives:
///    - [`SyncEntry::Sync`] or [`SyncEntry::UpcallDesc`] → returned to caller.
///    - Any other entry (Layer-1 event or SyscallDesc) → forwarded to `dispatch`.
/// 3. Panics if `mine` was successfully pushed but no peer reply arrives within
///    [`Transport::SYNC_TIMEOUT_TICKS`] — indicates divergence or SEU.
///
/// The caller is responsible for comparing `mine` against the returned entry
/// and panicking on mismatch; this keeps the primitive free of comparison
/// semantics so it can serve both Sync barriers and upcall descriptors.
pub fn lockstep_barrier<T: Transport>(
    transport: &T,
    mine: SyncEntry,
    dispatch: impl Fn(SyncEntry),
) -> SyncEntry {
    let mut pushed = false;
    let start = transport.now_ticks();
    loop {
        if !pushed && transport.try_push(mine) {
            transport.kick_peer();
            pushed = true;
        }
        if let Some(e) = transport.try_pop() {
            match e {
                SyncEntry::Sync { .. } | SyncEntry::UpcallDesc { .. } => return e,
                other => dispatch(other),
            }
        }
        if pushed && transport.now_ticks().wrapping_sub(start) >= T::SYNC_TIMEOUT_TICKS {
            panic!("lockstep: barrier timeout (possible SEU or divergence)");
        }
        transport.on_spin();
        core::hint::spin_loop();
    }
}

// ---------------------------------------------------------------------------
// Pending syscall queue (shadow side)
// ---------------------------------------------------------------------------

/// SPSC queue of [`SyscallDesc`] entries buffered for Phase-2 comparison.
///
/// Capacity matches the per-direction channel depth (4) so every descriptor
/// that fits in the channel also fits here.
///
/// Producer: shadow Phase-1 drain via [`store_pending_syscall`].
/// Consumer: shadow Phase-2 [`LockstepDriver::command`] via
///   [`take_pending_syscall`].
///
/// Both sides run on the shadow core sequentially (Phase 1 completes before
/// Phase 2 starts), so there is no concurrent access — the SPSC invariant
/// holds trivially.
static PENDING_SYSCALLS_SHADOW: SpscChannel<4, SyscallDesc> = SpscChannel::new();

/// Enqueue a [`SyscallDesc`] received from the leader for comparison in
/// Phase 2. Called by the board's Phase-1 drain loop (or `dispatch_layer1_event`).
pub fn store_pending_syscall(desc: SyscallDesc) {
    while !PENDING_SYSCALLS_SHADOW.push(desc) {
        core::hint::spin_loop();
    }
}

/// Dequeue the next pending [`SyscallDesc`]. Called by [`LockstepDriver`] in
/// Phase 2 on the shadow core.
pub(crate) fn take_pending_syscall() -> Option<SyscallDesc> {
    PENDING_SYSCALLS_SHADOW.pop()
}

// ---------------------------------------------------------------------------
// LockstepDriver — Layer-2 syscall interceptor
// ---------------------------------------------------------------------------

/// Wraps any [`kernel::syscall::SyscallDriver`] and exchanges a
/// [`SyncEntry::SyscallDesc`] on each `command()` call.
///
/// - **Leader (core 0)**: pushes a descriptor (with optional payload
///   fingerprint) to the transport channel, kicks the shadow, then blocks
///   until the shadow echoes the same descriptor back as confirmation. Only
///   then calls `inner.command()` — this is the **before-emit gate**.
/// - **Shadow (core 1)**: pops the descriptor buffered by
///   [`store_pending_syscall`] during Phase 1, compares scalar args and
///   payload fingerprint (panicking on mismatch), echoes the descriptor back,
///   then calls `inner.command()`.
///
/// Set `payload_allow_num` (via board wiring) to fingerprint the app's
/// RO-allow buffer before each gated command. For the console driver, slot 1
/// is the TX buffer (`ro_allow::WRITE`).
pub struct LockstepDriver<'a, T: Transport, D: kernel::syscall::SyscallDriver + kernel::syscall::LockstepPayload> {
    transport: &'a T,
    inner: &'a D,
    driver_num: usize,
}

impl<'a, T: Transport, D: kernel::syscall::SyscallDriver + kernel::syscall::LockstepPayload>
    LockstepDriver<'a, T, D>
{
    pub fn new(transport: &'a T, inner: &'a D, driver_num: usize) -> Self {
        Self { transport, inner, driver_num }
    }
}

impl<T: Transport, D: kernel::syscall::SyscallDriver + kernel::syscall::LockstepPayload>
    kernel::syscall::SyscallDriver for LockstepDriver<'_, T, D>
{
    fn command(
        &self,
        cmd: usize,
        arg0: usize,
        arg1: usize,
        processid: kernel::ProcessId,
    ) -> kernel::syscall::CommandReturn {
        let payload_fp = self.inner.command_payload_fp(cmd, processid);
        let desc = SyscallDesc {
            driver_num: self.driver_num as u32,
            sub: cmd as u8,
            arg0: arg0 as u32,
            arg1: arg1 as u32,
            payload_fp,
        };

        if self.transport.core_id() == 0 {
            // Leader path: push descriptor, kick shadow, then block until
            // the shadow echoes the same descriptor back (before-emit gate).
            while !self.transport.try_push(SyncEntry::SyscallDesc(desc)) {
                core::hint::spin_loop();
            }
            self.transport.kick_peer();

            let start = self.transport.now_ticks();
            loop {
                match self.transport.try_pop() {
                    Some(SyncEntry::SyscallDesc(SyscallDesc {
                        driver_num: d,
                        sub: s,
                        arg0: a0,
                        arg1: a1,
                        payload_fp: fp,
                    })) => {
                        if d != desc.driver_num
                            || s != desc.sub
                            || a0 != desc.arg0
                            || a1 != desc.arg1
                            || fp != desc.payload_fp
                        {
                            panic!(
                                "Lockstep Layer-2 gate: descriptor mismatch driver {}: \
                                 leader=(sub={},a0={:#x},a1={:#x},fp={:#010x}) \
                                 shadow=(sub={s},a0={a0:#x},a1={a1:#x},fp={fp:#010x})",
                                self.driver_num,
                                desc.sub,
                                desc.arg0,
                                desc.arg1,
                                desc.payload_fp,
                            );
                        }
                        break;
                    }
                    Some(_unexpected) => panic!(
                        "Lockstep Layer-2 gate: unexpected channel entry while waiting \
                         for shadow confirmation (driver {}, sub {cmd})",
                        self.driver_num,
                    ),
                    None => {}
                }
                self.transport.on_spin();
                if self.transport.now_ticks().wrapping_sub(start) >= T::SYNC_TIMEOUT_TICKS {
                    panic!(
                        "Lockstep Layer-2 gate: timeout waiting for shadow \
                         (driver {}, sub {cmd})",
                        self.driver_num,
                    );
                }
                core::hint::spin_loop();
            }
        } else {
            // Shadow path: pop the leader's descriptor (stored during Phase-1
            // drain), compare, echo it back as confirmation, then dispatch.
            match take_pending_syscall() {
                Some(SyscallDesc {
                    driver_num: d,
                    sub: s,
                    arg0: a0,
                    arg1: a1,
                    payload_fp: fp,
                }) => {
                    if d != self.driver_num as u32
                        || s != cmd as u8
                        || a0 != arg0 as u32
                        || a1 != arg1 as u32
                        || fp != payload_fp
                    {
                        panic!(
                            "Lockstep Layer-2: syscall mismatch driver {}: \
                             leader=(d={d},sub={s},a0={a0:#x},a1={a1:#x},fp={fp:#010x}) \
                             shadow=(sub={cmd},a0={arg0:#x},a1={arg1:#x},fp={payload_fp:#010x})",
                            self.driver_num,
                        );
                    }
                    // Echo our descriptor back as confirmation. The leader is
                    // spin-polling try_pop, so no kick needed.
                    while !self.transport.try_push(SyncEntry::SyscallDesc(desc)) {
                        core::hint::spin_loop();
                    }
                }
                None => panic!(
                    "Lockstep Layer-2: shadow driver {} sub {cmd} has no matching leader descriptor",
                    self.driver_num,
                ),
            }
        }
        self.inner.command(cmd, arg0, arg1, processid)
    }

    fn allocate_grant(&self, processid: kernel::ProcessId) -> Result<(), kernel::process::Error> {
        self.inner.allocate_grant(processid)
    }
}

// ---------------------------------------------------------------------------
// Registry types (defined here; initialized by the board with capsule consts)
// ---------------------------------------------------------------------------

/// How a particular upcall is verified across cores.
pub enum UpcallMode {
    /// Both cores generate this upcall independently (e.g. console WRITE_DONE
    /// after a Layer-1 TX replay). Compare the masked fields and panic on
    /// mismatch.
    Compare,
    /// The leader holds the authoritative value for one or more masked fields
    /// (e.g. the live `now` timestamp in an alarm upcall). The shadow uses the
    /// leader's value for those fields and its own for the rest.
    Forward,
}

/// Verification rule for a single upcall subscription slot.
pub struct UpcallRule {
    /// Subscribe number (slot within the driver, 0-indexed).
    pub subscribe_num: usize,
    /// Compare or forward semantics (see [`UpcallMode`]).
    pub mode: UpcallMode,
    /// Which of `(r0, r1, r2)` to compare/forward (`true`) vs ignore (`false`).
    /// Unmasked fields are excluded from comparison and left at their
    /// per-core values.
    pub mask: (bool, bool, bool),
}

/// Verification rules for all intercepted upcalls of a single driver.
pub struct DriverUpcallRules {
    /// Driver number (`capsules_core::xxx::DRIVER_NUM`), set by the board.
    pub driver_num: usize,
    /// Rules for each subscribe slot that needs cross-core verification.
    pub rules: &'static [UpcallRule],
}

// ---------------------------------------------------------------------------
// UartHooks — per-HIL sync logic for UART
// ---------------------------------------------------------------------------

/// Callbacks invoked by [`LockstepUart`] at each UART boundary.
///
/// Implement this trait to express the lockstep sync logic for a given
/// platform. [`LockstepUart`] handles all HIL trait plumbing; the hooks
/// carry the transport-specific work (channel pushes, kicks, SRAM copies).
pub trait UartHooks {
    /// Called just before `transmit_buffer` is forwarded to the inner driver.
    ///
    /// Use for Layer-2 verify+gate (fingerprint the payload, exchange with peer,
    /// compare). Currently a no-op in `QemuUartHooks` (Stage 1 TODO).
    fn on_transmit(&self, buf: &[u8]);

    /// Called in `transmitted_buffer` (upward callback) before the capsule
    /// client is notified.
    ///
    /// On the leader: push `UartTxDone` to the channel and kick the shadow.
    /// On the shadow: no-op (the replay mechanism drives callbacks).
    fn on_transmitted(&self, buf: &[u8]);

    /// Called in `received_buffer` (upward callback) before the capsule
    /// client is notified.
    ///
    /// On the leader: copy data to the [`BulkTag::UartRx`] shared-SRAM slot,
    /// push `UartRxReady { len }` to the channel, and kick the shadow.
    /// On the shadow: no-op.
    fn on_received(&self, buf: &[u8]);
}

// ---------------------------------------------------------------------------
// LockstepUart — generic HIL wrapper
// ---------------------------------------------------------------------------

/// Wraps any UART inner driver `U` with lockstep hooks `H`.
///
/// The leader wires `LockstepUart<RealUart, PlatformUartHooks>` between its
/// real UART peripheral and the `MuxUart`; the shadow wires
/// `LockstepUart<VirtualUartBuffer, PlatformUartHooks>` between its replay
/// buffer and the `Console`. The board registers:
///
/// - `inner` as calling the wrapper's [`TransmitClient`] / [`ReceiveClient`]
///   (upward path: inner fires → wrapper hook → capsule).
/// - the wrapper as calling the capsule's [`TransmitClient`] / [`ReceiveClient`].
pub struct LockstepUart<'a, U, H> {
    inner: &'a U,
    hooks: &'a H,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
}

impl<'a, U, H> LockstepUart<'a, U, H> {
    pub fn new(inner: &'a U, hooks: &'a H) -> Self {
        Self {
            inner,
            hooks,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl<'a, U: hil::uart::Configure, H: UartHooks> hil::uart::Configure
    for LockstepUart<'a, U, H>
{
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), kernel::ErrorCode> {
        self.inner.configure(params)
    }
}

impl<'a, U: hil::uart::Transmit<'a>, H: UartHooks> hil::uart::Transmit<'a>
    for LockstepUart<'a, U, H>
{
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buf: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        self.hooks.on_transmit(&tx_buf[..tx_len]);
        self.inner.transmit_buffer(tx_buf, tx_len)
    }

    fn transmit_word(&self, word: u32) -> Result<(), kernel::ErrorCode> {
        self.inner.transmit_word(word)
    }

    fn transmit_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.inner.transmit_abort()
    }
}

impl<'a, U: hil::uart::Receive<'a>, H: UartHooks> hil::uart::Receive<'a>
    for LockstepUart<'a, U, H>
{
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buf: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        self.inner.receive_buffer(rx_buf, rx_len)
    }

    fn receive_word(&self) -> Result<(), kernel::ErrorCode> {
        self.inner.receive_word()
    }

    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.inner.receive_abort()
    }
}

impl<'a, U, H: UartHooks> hil::uart::TransmitClient for LockstepUart<'a, U, H> {
    fn transmitted_buffer(
        &self,
        tx_buf: &'static mut [u8],
        tx_len: usize,
        rcode: Result<(), kernel::ErrorCode>,
    ) {
        self.hooks.on_transmitted(&tx_buf[..tx_len]);
        self.tx_client
            .map(|c| c.transmitted_buffer(tx_buf, tx_len, rcode));
    }

    fn transmitted_word(&self, rcode: Result<(), kernel::ErrorCode>) {
        self.tx_client.map(|c| c.transmitted_word(rcode));
    }
}

impl<'a, U, H: UartHooks> hil::uart::ReceiveClient for LockstepUart<'a, U, H> {
    fn received_buffer(
        &self,
        rx_buf: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), kernel::ErrorCode>,
        error: hil::uart::Error,
    ) {
        self.hooks.on_received(&rx_buf[..rx_len]);
        self.rx_client
            .map(|c| c.received_buffer(rx_buf, rx_len, rcode, error));
    }

    fn received_word(
        &self,
        word: u32,
        rcode: Result<(), kernel::ErrorCode>,
        error: hil::uart::Error,
    ) {
        self.rx_client.map(|c| c.received_word(word, rcode, error));
    }
}
