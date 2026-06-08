// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mode Entry (MC_ME), Reset Generation Module (MC_RGM), and Reset Domain
//! Controller (RDC) driver for NXP S32G3.
//!
//! Register definitions and bitfields are taken from the S32G3 Reference
//! Manual, Chapter 33 (MC_ME), Chapter 30 (MC_RGM), and Chapter 29
//! (Reset / RDC).
//!
//! The driver exposes the [`partition_enable`] entry point that brings a
//! software-resettable partition out of reset and turns its peripheral clocks
//! on. The flow follows RM §29.4 and RM §33.5: enable partition clock
//! (PCONF/PUPD), release the RDC interconnect, deassert the MC_RGM peripheral
//! reset, then clear output-safe-state. See [`partition_enable`] for the
//! exact step ordering.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

// Memory map from RM §33.4.1 (MC_ME) and RM §29.13.1 (RDC). The MC_ME
// partition registers repeat every 0x200 bytes; four partitions are exposed
// in the linear array, matching the S32G3 reset domain count.
register_structs! {
    /// MC_ME register block. See RM §33.4.1 for the full memory map; only the
    /// subset used by the partition turn-on flow is declared.
    pub McMeRegisters {
        /// Control Key Register
        /// RM §33.4.2.
        (0x000 => ctl_key: ReadWrite<u32, CTL_KEY::Register>),
        /// Reserved gap between CTL_KEY and the partition register block
        /// (RM §33.4.1 memory map: partition 0 starts at offset 100h).
        (0x004 => _reserved0),
        /// Per-partition register block. Four entries cover partitions 0..3
        /// (RM §33.4.7..§33.4.74).
        (0x100 => partitions: [PartitionRegisters; 4]),
        (0x100 + 4 * 0x200 => @END),
    }
}

register_structs! {
    /// Per-partition MC_ME register block. RM §33.4.7 (PCONF), §33.4.8
    /// (PUPD), §33.4.9 (STAT).
    pub PartitionRegisters {
        /// Partition n Process Configuration Register
        /// RM §33.4.7 (partition 0), §33.4.32 (partition 1), §33.4.67
        /// (partition 2), §33.4.72 (partition 3).
        (0x000 => pconf: ReadWrite<u32, PCONF::Register>),
        /// Partition n Process Update Register
        /// RM §33.4.8 (partition 0), §33.4.33 (partition 1), §33.4.68
        /// (partition 2), §33.4.73 (partition 3).
        (0x004 => pupd: ReadWrite<u32, PUPD::Register>),
        /// Partition n Status Register
        /// RM §33.4.9 (partition 0), §33.4.34 (partition 1), §33.4.69
        /// (partition 2), §33.4.74 (partition 3).
        (0x008 => stat: ReadWrite<u32, STAT::Register>),
        /// Reserved padding to reach the next partition's 0x200-byte slot
        /// (RM §33.4.1 memory map).
        (0x00C => _reserved),
        (0x200 => @END),
    }
}

register_structs! {
    /// MC_RGM register block. RM §30.7.1 memory map; only the Peripheral
    /// Reset / Status sections are declared.
    pub McRgmRegisters {
        /// Reserved 0x40-byte header before the PRST array
        /// (RM §30.7.1 memory map: PRST0_0 lives at offset 40h).
        (0x000 => _reserved0),
        /// Peripheral Reset registers PRST0_0..PRST3_0
        /// RM §30.7.10..§30.7.13.
        (0x040 => prst: [PartitionResetRegisters; 4]),
        /// Reserved gap between PRST and PSTAT blocks
        /// (RM §30.7.1 memory map).
        (0x040 + 4 * 8 => _reserved1),
        /// Peripheral Reset Status registers PSTAT0_0..PSTAT3_0
        /// RM §30.7.14..§30.7.17.
        (0x140 => pstat: [PartitionStatusRegisters; 4]),
        (0x140 + 4 * 8 => @END),
    }
}

register_structs! {
    /// MC_RGM PRSTn_0 — peripheral reset control for partition n.
    /// RM §30.7.10..§30.7.13.
    pub PartitionResetRegisters {
        /// Peripheral Reset register
        /// RM §30.7.10 (partition 0), §30.7.11 (partition 1), §30.7.12
        /// (partition 2), §30.7.13 (partition 3).
        (0x000 => rst: ReadWrite<u32, RGM_PRST::Register>),
        /// Reserved padding to reach the 8-byte partition slot
        /// (RM §30.7.1 memory map).
        (0x004 => _reserved),
        (0x008 => @END),
    }
}

register_structs! {
    /// MC_RGM PSTATn_0 — peripheral reset status for partition n.
    /// RM §30.7.14..§30.7.17.
    pub PartitionStatusRegisters {
        /// Peripheral Reset Status register
        /// RM §30.7.14 (partition 0), §30.7.15 (partition 1), §30.7.16
        /// (partition 2), §30.7.17 (partition 3).
        (0x000 => stat: ReadWrite<u32, RGM_PSTAT::Register>),
        /// Reserved padding to reach the 8-byte partition slot
        /// (RM §30.7.1 memory map).
        (0x004 => _reserved),
        (0x008 => @END),
    }
}

register_structs! {
    /// RDC (Reset Domain Controller) register block. RM §29.13.1.
    pub RdcRegisters {
        /// Software Reset Domain n Control registers RD1_CTRL_REG..RD3_CTRL_REG
        /// RM §29.13.2..§29.13.4.
        (0x000 => rd_ctrl: [ReadWrite<u32, RDC_CTRL::Register>; 32]),
        /// Software Reset Domain n Status registers RD1_STAT_REG..RD3_STAT_REG
        /// RM §29.13.5..§29.13.7.
        (0x080 => rd_status: [ReadWrite<u32, RDC_STATUS::Register>; 32]),
        (0x100 => @END),
    }
}

register_bitfields![u32,
    /// Control Key Register
    /// RM §33.4.2.
    /// Provides the hardware process trigger for the MC_ME state machine.
    /// Two writes are required: first with the key `0x5AF0`, then with the
    /// inverted key `0xA50F` (RM §33.4.2).
    pub CTL_KEY [
        /// Reserved. Read returns 0 (RM §33.4.2 field `31-16`).
        _RSV_16_31 OFFSET(16) NUMBITS(16) [],
        /// Control key. Magic numbers that trigger an update of the MC_ME
        /// state machine after changing partition configuration
        /// (RM §33.4.2 field `15-0 KEY`).
        KEY OFFSET(0) NUMBITS(16) [
            /// First key: `0x5AF0` (RM §33.4.2).
            TRIGGER_1 = 0x5AF0,
            /// Second (inverted) key: `0xA50F` (RM §33.4.2).
            TRIGGER_2 = 0xA50F,
        ]
    ],

    /// Partition n Process Configuration Register
    /// RM §33.4.7 (partition 0), §33.4.32 (partition 1), §33.4.67
    /// (partition 2), §33.4.72 (partition 3).
    /// Holds the per-process enable bits; the actual transition is triggered
    /// by writing the corresponding bit in PUPD followed by the CTL_KEY
    /// magic sequence (RM §33.5).
    pub PCONF [
        /// Reserved. Read returns 0 (RM §33.4.7 field `31-3`).
        _RSV_3_31 OFFSET(3) NUMBITS(29) [],
        /// Output Safe Stating Enable. Configures whether the partition
        /// outputs are forced to their safe state. Bit is present on
        /// partitions 1..7 only (RM §33.4.32 field `2 OSSE`).
        OSSE OFFSET(2) NUMBITS(1) [
            /// Disable output safe stating.
            Disabled = 0,
            /// Enable output safe stating.
            Enabled = 1,
        ],
        /// Reserved. Read returns 0 (RM §33.4.7 field `1`).
        _RSV_1_1 OFFSET(1) NUMBITS(1) [],
        /// Partition Clock Enable. Controls whether the clock to IPs in the
        /// partition (other than cores) is enabled
        /// (RM §33.4.7 field `0 PCE`).
        PCE  OFFSET(0) NUMBITS(1) [
            /// Disable the clock to IPs.
            Disabled = 0,
            /// Enable the clock to IPs.
            Enabled = 1,
        ]
    ],

    /// Partition n Process Update Register
    /// RM §33.4.8 (partition 0), §33.4.33 (partition 1), §33.4.68
    /// (partition 2), §33.4.73 (partition 3).
    /// Each bit acts as a trigger for the corresponding hardware process
    /// described by the matching PCONF field. Bits are auto-cleared by
    /// hardware once the process completes (RM §33.4.8).
    pub PUPD [
        /// Reserved. Read returns 0 (RM §33.4.8 field `31-3`).
        _RSV_3_31 OFFSET(3) NUMBITS(29) [],
        /// Output Safe Stating Update. Triggers the hardware process for
        /// enabling/disabling output safe stating. Bit is present on
        /// partitions 1..7 only (RM §33.4.33 field `2 OSSUD`).
        OSSUD OFFSET(2) NUMBITS(1) [
            /// Do not trigger the hardware process.
            NoTrigger = 0,
            /// Trigger the hardware process.
            Trigger = 1,
        ],
        /// Reserved. Read returns 0 (RM §33.4.8 field `1`).
        _RSV_1_1 OFFSET(1) NUMBITS(1) [],
        /// Partition Clock Update. Triggers the partition clock enable /
        /// disable hardware process (RM §33.4.8 field `0 PCUD`).
        PCUD  OFFSET(0) NUMBITS(1) [
            /// Do not trigger the hardware process.
            NoTrigger = 0,
            /// Trigger the hardware process.
            Trigger = 1,
        ]
    ],

    /// Partition n Status Register
    /// RM §33.4.9 (partition 0), §33.4.34 (partition 1), §33.4.69
    /// (partition 2), §33.4.74 (partition 3).
    /// Reflects the current state of the partition's control signals.
    pub STAT [
        /// Reserved. Read returns 0 (RM §33.4.9 field `31-3`).
        _RSV_3_31 OFFSET(3) NUMBITS(29) [],
        /// Output Safe Stating Status. Bit is present on partitions 1..7
        /// only (RM §33.4.34 field `2 OSSS`).
        OSSS OFFSET(2) NUMBITS(1) [
            /// Output safe stating is not active.
            Inactive = 0,
            /// Output safe stating is active.
            Active = 1,
        ],
        /// Reserved. Read returns 0 (RM §33.4.9 field `1`).
        _RSV_1_1 OFFSET(1) NUMBITS(1) [],
        /// Partition Clock Status. Indicates whether the partition clock is
        /// active (RM §33.4.9 field `0 PCS`).
        PCS  OFFSET(0) NUMBITS(1) [
            /// Clock is inactive.
            Inactive = 0,
            /// Clock is active.
            Active = 1,
        ]
    ],

    /// MC_RGM Peripheral Reset register
    /// RM §30.7.10 (PRST0_0), §30.7.11 (PRST1_0), §30.7.12 (PRST2_0),
    /// §30.7.13 (PRST3_0).
    pub RGM_PRST [
        /// Peripheral Reset. Each `PERIPH_n_RST` bit controls the reset
        /// state of one peripheral; bit `n` corresponds to `PERIPH_n_RST`
        /// (RM §30.7.10 field `0 PERIPH_n_RST`).
        PERIPH_RST OFFSET(0) NUMBITS(1) [
            /// No forced reset on the peripheral.
            Released = 0,
            /// Forced reset on the peripheral.
            Asserted = 1,
        ]
    ],

    /// MC_RGM Peripheral Reset Status register
    /// RM §30.7.14 (PSTAT0_0), §30.7.15 (PSTAT1_0), §30.7.16 (PSTAT2_0),
    /// §30.7.17 (PSTAT3_0).
    pub RGM_PSTAT [
        /// Peripheral Reset Status. Bit is set when a peripheral's reset is
        /// still asserted (RM §30.7.14 field `0 PERIPH_n_RST`).
        PERIPH_RST_STAT OFFSET(0) NUMBITS(1) [
            /// No reset asserted.
            Released = 0,
            /// Reset is asserted.
            Asserted = 1,
        ]
    ],

    /// RDC Software Reset Domain n Control Register
    /// RM §29.13.2 (RD1_CTRL_REG), §29.13.3 (RD2_CTRL_REG), §29.13.4
    /// (RD3_CTRL_REG).
    pub RDC_CTRL [
        /// Control Register Unlock. The control register must be unlocked
        /// before any other field is updated; writing 0 re-locks
        /// (RM §29.13.2 field `31 UNLOCK`).
        UNLOCK            OFFSET(31) NUMBITS(1) [
            /// Control register fields are locked and cannot be updated
            /// (except for this field).
            Locked = 0,
            /// Control register fields are unlocked and can be updated.
            Unlocked = 1,
        ],
        /// Interconnect Interface Disable. Disables the partition's
        /// interconnect interface (RM §29.13.2 field `3 INTERCONNECT_DIS`).
        INTERCONNECT_DIS  OFFSET(3)  NUMBITS(1) [
            /// Enable interconnect interface.
            Enabled = 0,
            /// Disable interconnect interface.
            Disabled = 1,
        ]
    ],

    /// RDC Software Reset Domain n Status Register
    /// RM §29.13.5 (RD1_STAT_REG), §29.13.6 (RD2_STAT_REG), §29.13.7
    /// (RD3_STAT_REG).
    pub RDC_STATUS [
        /// Interconnect Interface Disable Status. Acknowledges that the
        /// interconnect interface is disabled
        /// (RM §29.13.5 field `4 INTERCONNECT_DIS_STAT`).
        INTERCONNECT_STAT OFFSET(4) NUMBITS(1) [
            /// Interconnect interface is not disabled.
            Active = 0,
            /// Interconnect interface is disabled.
            Inactive = 1,
        ]
    ]
];

// Base addresses from the S32G3 memory map.

/// Base address of MC_ME instance.
pub const MC_ME_BASE: StaticRef<McMeRegisters> =
    unsafe { StaticRef::new(0x4008_8000 as *const McMeRegisters) };

/// Base address of MC_RGM instance.
pub const MC_RGM_BASE: StaticRef<McRgmRegisters> =
    unsafe { StaticRef::new(0x4007_8000 as *const McRgmRegisters) };

/// Base address of RDC instance.
pub const RDC_BASE: StaticRef<RdcRegisters> =
    unsafe { StaticRef::new(0x4008_0000 as *const RdcRegisters) };

/// Bring a software-resettable partition out of reset and turn its
/// peripheral clocks on.
///
/// `part` selects the partition index (0..3 on the S32G3). The flow
/// follows RM §29.4 and RM §33.5:
///
/// 1. Enable the partition clock and update the MC_ME state machine
///    (PCONF/PUPD then CTL_KEY magic sequence).
/// 2. Release the RDC interconnect for the partition.
/// 3. Deassert the MC_RGM peripheral reset for the partition.
/// 4. Clear output safe stating and wait for the reset status to clear.
///
/// The function is a no-op when the partition is already enabled (PCE
/// already reflected in PCS).
pub fn partition_enable(part: usize) {
    assert!(part < 4, "S32G3 only has 4 partitions");
    let mc_me = MC_ME_BASE;
    let mc_rgm = MC_RGM_BASE;
    let rdc = RDC_BASE;

    // Already enabled — PCS reflects PCE having taken effect
    // (RM §33.4.9 field `0 PCS`).
    if mc_me.partitions[part].stat.is_set(STAT::PCS) {
        return;
    }

    // 1. Enable partition clock (RM §33.4.7 PCE, §33.4.8 PCUD).
    mc_me.partitions[part].pconf.modify(PCONF::PCE::SET);
    mc_me.partitions[part].pupd.modify(PUPD::PCUD::SET);

    // Trigger and wait
    mc_me_trigger();
    mc_me_wait(part, false);

    // 2. Enable RDC interconnect (RM §29.13.2 UNLOCK / INTERCONNECT_DIS).
    if rdc.rd_status[part].is_set(RDC_STATUS::INTERCONNECT_STAT) {
        // already enabled
    } else {
        rdc.rd_ctrl[part].modify(RDC_CTRL::UNLOCK::SET);
        rdc.rd_ctrl[part].modify(RDC_CTRL::INTERCONNECT_DIS::CLEAR);
        // RM §29.13.5: INTERCONNECT_STAT clears when the interconnect is
        // fully re-enabled.
        while rdc.rd_status[part].is_set(RDC_STATUS::INTERCONNECT_STAT) {}
        rdc.rd_ctrl[part].modify(RDC_CTRL::UNLOCK::CLEAR);
    }

    // 3. Deassert peripheral reset (RM §30.7.10 PERIPH_RST).
    mc_rgm.prst[part].rst.modify(RGM_PRST::PERIPH_RST::CLEAR);

    // 4. Clear output-safe-state (RM §33.4.32 OSSE, §33.4.33 OSSUD),
    //    trigger update, and wait for the reset status to clear
    //    (RM §30.7.14 PERIPH_RST_STAT).
    mc_me.partitions[part].pconf.modify(PCONF::OSSE::CLEAR);
    mc_me.partitions[part].pupd.modify(PUPD::OSSUD::SET);
    mc_me_trigger();
    mc_me_wait(part, true);

    while mc_rgm.pstat[part].stat.is_set(RGM_PSTAT::PERIPH_RST_STAT) {}
}

/// Issue the CTL_KEY magic sequence that triggers the MC_ME state machine
/// (RM §33.4.2: key `0x5AF0` followed by inverted key `0xA50F`).
fn mc_me_trigger() {
    MC_ME_BASE.ctl_key.write(CTL_KEY::KEY::TRIGGER_1);
    MC_ME_BASE.ctl_key.write(CTL_KEY::KEY::TRIGGER_2);
}

/// Poll the partition status register until the requested state has been
/// reached. `is_osse` selects between waiting for the partition clock
/// transition (`false`, observes PCS) and the output-safe-state transition
/// (`true`, observes OSSS). See RM §33.4.9 and §33.4.34.
/// This function must only be called from init code.
fn mc_me_wait(part: usize, is_osse: bool) {
    const MAX_WAIT_CYCLES: usize = 1_000_000;
    let mc_me = MC_ME_BASE;

    // This is okay to poll here, as we are in board initialization code.
    for _ in 0..MAX_WAIT_CYCLES {
        if is_osse {
            let want = mc_me.partitions[part].pconf.is_set(PCONF::OSSE);
            if mc_me.partitions[part].stat.is_set(STAT::OSSS) == want {
                return;
            }
        } else {
            let want = mc_me.partitions[part].pconf.is_set(PCONF::PCE);
            if mc_me.partitions[part].stat.is_set(STAT::PCS) == want {
                return;
            }
        }
    }
    // The hardware should have completed the transition well before this point;
    // There is not much we can do if it hasn't, so panic with a message.
    panic!(
        "MC_ME wait timed out on partition {} is_osse: {}",
        part, is_osse
    );
}
