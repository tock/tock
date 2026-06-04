// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Software Watchdog Timer (SWT) driver for NXP S32G3.
//!
//! Register definitions and bitfields are taken from the S32G3 Reference
//! Manual, Chapter 42. The S32G3 has one SWT instance per boot-capable core;
//! SWT_0 is the boot-target watchdog for Cortex-M7 core 0. This driver maps the
//! full SWT programming model and currently only supports disabling the timer.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// Base address of SWT_0, the watchdog instance for Cortex-M7 core 0.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_0 at 0x4010_0000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_0_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4010_0000 as *const SwtRegisters) };

/// Base address of SWT_1, the watchdog instance for Cortex-M7 core 1.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_1 at 0x4010_4000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_1_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4010_4000 as *const SwtRegisters) };

/// Base address of SWT_2, the watchdog instance for Cortex-M7 core 2.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_2 at 0x4010_8000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_2_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4010_8000 as *const SwtRegisters) };

/// Base address of SWT_3, the watchdog instance for A53 cluster 0 core 0.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_3 at 0x4010_C000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_3_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4010_C000 as *const SwtRegisters) };

/// Base address of SWT_4, the watchdog instance for A53 cluster 0 core 1.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_4 at 0x4020_0000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_4_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4020_0000 as *const SwtRegisters) };

/// Base address of SWT_5, the watchdog instance for A53 cluster 1 core 0.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_5 at 0x4020_4000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_5_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4020_4000 as *const SwtRegisters) };

/// Base address of SWT_6, the watchdog instance for A53 cluster 1 core 1.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_6 at 0x4020_8000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_6_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4020_8000 as *const SwtRegisters) };

/// Base address of SWT_7, the watchdog instance for Cortex-M7 core 3.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_7 at 0x4020_C000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_7_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4020_C000 as *const SwtRegisters) };

/// Base address of SWT_8, the watchdog instance for A53 cluster 0 core 2.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_8 at 0x4050_0000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_8_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4050_0000 as *const SwtRegisters) };

/// Base address of SWT_9, the watchdog instance for A53 cluster 0 core 3.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_9 at 0x4050_4000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_9_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4050_4000 as *const SwtRegisters) };

/// Base address of SWT_10, the watchdog instance for A53 cluster 1 core 2.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_10 at 0x4050_8000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_10_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4050_8000 as *const SwtRegisters) };

/// Base address of SWT_11, the watchdog instance for A53 cluster 1 core 3.
/// ### Safety: The S32G3 RM §42.6.1 maps SWT_11 at 0x4050_C000, and this
/// `StaticRef` is only used for volatile MMIO access through `SwtRegisters`.
pub const SWT_11_BASE: StaticRef<SwtRegisters> =
    unsafe { StaticRef::new(0x4050_C000 as *const SwtRegisters) };

register_structs! {
    pub SwtRegisters {
        /// Control Register — configures SWT enable, locks, service mode,
        /// timeout reaction, debug/stop behavior, invalid-access behavior, and
        /// master access protection (RM §42.6.2).
        (0x000 => pub cr: ReadWrite<u32, CR::Register>),
        /// Interrupt Register — timeout interrupt flag; write one to clear
        /// `TIF` (RM §42.6.3).
        (0x004 => pub ir: ReadWrite<u32, IR::Register>),
        /// Timeout Register — watchdog timeout period in counter clock cycles
        /// (RM §42.6.4).
        (0x008 => pub to: ReadWrite<u32, TO::Register>),
        /// Window Register — window start value for window service mode
        /// (RM §42.6.5).
        (0x00C => pub wn: ReadWrite<u32, WN::Register>),
        /// Service Register — accepts the fixed/keyed service sequence and the
        /// soft-unlock sequence; reads return zero (RM §42.6.6).
        (0x010 => pub sr: ReadWrite<u32, SR::Register>),
        /// Counter Output Register — captures the internal timer when SWT is
        /// disabled (RM §42.6.7).
        (0x014 => pub co: ReadOnly<u32, CO::Register>),
        /// Service Key Register — holds the previous/initial key for keyed
        /// service mode (RM §42.6.8).
        (0x018 => pub sk: ReadWrite<u32, SK::Register>),
        /// Event Request Register — timeout reset request flag; write one to
        /// clear `RRF` and the request (RM §42.6.9).
        (0x01C => pub rrr: ReadWrite<u32, RRR::Register>),
        (0x020 => @END),
    }
}

register_bitfields![u32,
    /// Control Register (CR), RM §42.6.2.
    pub CR [
        /// Master Access Protection 0. Controls bus masters with XRDC DIDs 0
        /// and 8.
        MAP0 OFFSET(31) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 1. Controls bus masters with XRDC DIDs 1
        /// and 9.
        MAP1 OFFSET(30) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 2. Controls bus masters with XRDC DIDs 2
        /// and 10.
        MAP2 OFFSET(29) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 3. Controls bus masters with XRDC DIDs 3
        /// and 11.
        MAP3 OFFSET(28) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 4. Controls bus masters with XRDC DIDs 4
        /// and 12.
        MAP4 OFFSET(27) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 5. Controls bus masters with XRDC DIDs 5
        /// and 13.
        MAP5 OFFSET(26) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 6. Controls bus masters with XRDC DIDs 6
        /// and 14.
        MAP6 OFFSET(25) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Master Access Protection 7. Controls bus masters with XRDC DIDs 7
        /// and 15.
        MAP7 OFFSET(24) NUMBITS(1) [
            /// Access disabled for this master-protection group.
            Disabled = 0,
            /// Access enabled for this master-protection group.
            Enabled = 1,
        ],
        /// Reserved. Reads return zero (RM §42.6.2 field `23-11`).
        _RSV_11_23 OFFSET(11) NUMBITS(13) [],
        /// Service Mode. Selects fixed or keyed watchdog service sequence.
        SMD OFFSET(9) NUMBITS(2) [
            /// Fixed service sequence: write 0xA602 then 0xB480 to SR.
            Fixed = 0,
            /// Keyed service sequence: write two pseudorandom keys derived from SK.
            Keyed = 1,
        ],
        /// Reset on Invalid Access. Selects bus-error-only or bus-error plus
        /// reset request behavior for invalid SWT accesses.
        RIA OFFSET(8) NUMBITS(1) [
            /// Invalid access generates a bus error.
            BusError = 0,
            /// Invalid access generates a bus error and, if enabled, a reset request.
            BusErrorAndReset = 1,
        ],
        /// Window Mode. Restricts servicing to the configured window when set.
        WND OFFSET(7) NUMBITS(1) [
            /// Service sequence may be executed at any time.
            Regular = 0,
            /// Service sequence is valid only when the counter is below WN.
            Window = 1,
        ],
        /// Interrupt Then Reset Request. Selects initial timeout interrupt vs
        /// immediate reset request behavior.
        ITR OFFSET(6) NUMBITS(1) [
            /// Generate a reset request on any timeout.
            Reset = 0,
            /// Generate an interrupt on the first timeout, reset on the next.
            InterruptThenReset = 1,
        ],
        /// Hard Lock. Makes CR, TO, WN, and SK read-only until reset when set.
        HLK OFFSET(5) NUMBITS(1) [
            /// Hard lock disabled.
            Unlocked = 0,
            /// Hard lock enabled until reset.
            Locked = 1,
        ],
        /// Soft Lock. Makes CR, TO, WN, and SK read-only until the unlock
        /// sequence is written to SR.
        SLK OFFSET(4) NUMBITS(1) [
            /// Soft lock disabled.
            Unlocked = 0,
            /// Soft lock enabled until SR receives 0xC520 then 0xD928.
            Locked = 1,
        ],
        /// Reserved. Reads return zero (RM §42.6.2 field `3`).
        _RSV_3 OFFSET(3) NUMBITS(1) [],
        /// Stop Mode Control. Selects watchdog behavior when the core enters
        /// Stop or Standby mode.
        STP OFFSET(2) NUMBITS(1) [
            /// Timer continues in Stop/Standby mode.
            Run = 0,
            /// Timer stops in Stop/Standby mode.
            Stop = 1,
        ],
        /// Debug Mode Control. Selects watchdog behavior while the core is in
        /// Debug mode.
        FRZ OFFSET(1) NUMBITS(1) [
            /// Timer continues in Debug mode.
            Run = 0,
            /// Timer stops in Debug mode.
            Stop = 1,
        ],
        /// Watchdog Enable. Starts or stops the SWT countdown timer.
        WEN OFFSET(0) NUMBITS(1) [
            /// Watchdog disabled.
            Disabled = 0,
            /// Watchdog enabled.
            Enabled = 1,
        ]
    ],

    /// Interrupt Register (IR), RM §42.6.3.
    pub IR [
        /// Reserved. Reads return zero (RM §42.6.3 field `31-1`).
        _RSV_1_31 OFFSET(1) NUMBITS(31) [],
        /// Timeout Interrupt Flag. Write one to clear the flag and interrupt.
        TIF OFFSET(0) NUMBITS(1) [
            /// No interrupt request due to an initial timeout.
            NoInterrupt = 0,
            /// Clear the timeout interrupt flag.
            Clear = 1,
        ]
    ],

    /// Timeout Register (TO), RM §42.6.4.
    pub TO [
        /// Watchdog Timeout. Timeout period in SWT counter clock cycles.
        WTO OFFSET(0) NUMBITS(32) []
    ],

    /// Window Register (WN), RM §42.6.5.
    pub WN [
        /// Window Start Value. In window mode, servicing is only valid when the
        /// internal timer is less than this value.
        WST OFFSET(0) NUMBITS(32) []
    ],

    /// Service Register (SR), RM §42.6.6.
    pub SR [
        /// Reserved. Reads return zero (RM §42.6.6 field `31-16`).
        _RSV_16_31 OFFSET(16) NUMBITS(16) [],
        /// Watchdog Service Code. Accepts fixed-service, keyed-service, and
        /// soft-unlock writes; reads return zero.
        WSC OFFSET(0) NUMBITS(16) [
            /// First fixed-service code.
            FixedFirst = 0xA602,
            /// Second fixed-service code.
            FixedSecond = 0xB480,
            /// First soft-unlock code.
            UnlockFirst = 0xC520,
            /// Second soft-unlock code.
            UnlockSecond = 0xD928,
        ]
    ],

    /// Counter Output Register (CO), RM §42.6.7.
    pub CO [
        /// Watchdog Count. Captured internal timer value when SWT is disabled.
        CNT OFFSET(0) NUMBITS(32) []
    ],

    /// Service Key Register (SK), RM §42.6.8.
    pub SK [
        /// Reserved. Reads return zero (RM §42.6.8 field `31-16`).
        _RSV_16_31 OFFSET(16) NUMBITS(16) [],
        /// Service Key. Holds the previous/initial key for keyed service mode.
        SK OFFSET(0) NUMBITS(16) []
    ],

    /// Event Request Register (RRR), RM §42.6.9.
    pub RRR [
        /// Reserved. Reads return zero (RM §42.6.9 field `31-1`).
        _RSV_1_31 OFFSET(1) NUMBITS(31) [],
        /// Reset Request Flag. Write one to clear the flag and request.
        RRF OFFSET(0) NUMBITS(1) [
            /// No reset request flag.
            NoRequest = 0,
            /// Clear the timeout reset request flag and request.
            Clear = 1,
        ]
    ]
];

/// Software Watchdog Timer (SWT) peripheral
///
/// This is a minimal driver for the S32G3's Software Watchdog Timer (SWT) peripheral. It
/// currently only supports disabling the timer, which is necessary to prevent unintended resets
/// as the watchdog is enabled by default.
pub struct Swt {
    registers: StaticRef<SwtRegisters>,
}

impl Swt {
    /// Creates a new SWT instance with the given register reference.
    pub const fn new(registers: StaticRef<SwtRegisters>) -> Self {
        Self { registers }
    }

    /// Returns whether the watchdog timer is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.registers.cr.is_set(CR::WEN)
    }

    /// Disables the watchdog timer.
    ///
    /// # INIT-ONLY
    /// The watchdog must be disabled before `kernel_loop()`. If called at runtime
    /// the WEN write races with a pending watchdog reset.
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// See safety manual §SWT-INIT.
    pub fn disable(&self) {
        self.unlock_soft_lock();
        self.registers.cr.modify(CR::WEN::Disabled);
    }

    /// Unlocks the soft lock by writing the unlock sequence to the status register.
    fn unlock_soft_lock(&self) {
        self.registers.sr.write(SR::WSC::UnlockFirst);
        self.registers.sr.write(SR::WSC::UnlockSecond);
    }
}

impl kernel::platform::watchdog::WatchDog for Swt {
    /// Sets up the watchdog by disabling it.
    ///
    /// # INIT-ONLY
    /// The watchdog must be disabled before `kernel_loop()`. If called at runtime
    /// the WEN write races with a pending watchdog reset.
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// See safety manual §SWT-INIT.
    fn setup(&self) {
        self.disable();
    }
}
