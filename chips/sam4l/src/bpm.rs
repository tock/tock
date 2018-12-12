//! Implementation of the BPM peripheral.

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

#[repr(C)]
struct BpmRegisters {
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
    sr: ReadOnly<u32, Status::Register>,
    unlock: ReadWrite<u32, Unlock::Register>,
    pmcon: ReadWrite<u32, PowerModeControl::Register>,
    _reserved0: [u32; 2],
    bkupwcause: ReadOnly<u32, BackupWakeup::Register>,
    bkupwen: ReadWrite<u32, BackupWakeup::Register>,
    bkuppmux: ReadWrite<u32, BackupPinMuxing::Register>,
    ioret: ReadWrite<u32, InputOutputRetention::Register>,
}

register_bitfields![u32,
    Interrupt [
        /// Access Error
        AE 31,
        /// Power Scaling OK
        PSOK 0
    ],

    Status [
        /// Access Error
        AE 31,
        /// Power Scaling OK
        PSOK 0
    ],

    Unlock [
        /// Unlock Key
        KEY OFFSET(24) NUMBITS(8) [],
        /// Unlock Address
        ADDR OFFSET(0) NUMBITS(10) []
    ],

    PowerModeControl [
        /// Fast Wakeup
        FASTWKUP OFFSET(24) NUMBITS(1) [
            NormalWakeup = 0,
            FastWakeup = 1
        ],
        /// 32kHz-1kHz Clock Source Selection
        CK32S OFFSET(16) NUMBITS(1) [
            Osc32k = 0,
            Rc32k = 1
        ],
        /// SLEEP mode Configuration
        SLEEP OFFSET(12) NUMBITS(2) [
            CpuStopped = 0,
            CpuAhbStopped = 1,
            CpuAhbPbGclkStopped = 2,
            CpuAhbPbGclkClockStopped = 3
        ],
        /// Retention Mode
        RET OFFSET(9) NUMBITS(1) [
            NoPowerSave = 0,
            PowerSave = 1
        ],
        /// Backup Mode
        BKUP OFFSET(8) NUMBITS(1) [
            NoPowerSave = 0,
            PowerSave = 1
        ],
        /// WARN: Undocumented!
        ///
        /// According to the datasheet (sec 6.2, p57) changing power scaling
        /// requires waiting for an interrupt (presumably because flash is
        /// inaccessible during the transition). However, the ASF code sets
        /// bit 3 ('PSCM' bit) of the PMCON register, which is *blank* (not a '-')
        /// in the datasheet with supporting comments that this allows a change
        /// 'without CPU halt'
        PSCM OFFSET(3) NUMBITS(1) [
            WithCpuHalt = 0,
            WithoutCpuHalt = 1
        ],
        /// Power Scaling Change Request
        PSCREQ OFFSET(2) NUMBITS(1) [
            PowerScalingNotRequested = 0,
            PowerScalingRequested = 1
        ],
        /// Power Scaling Configuration Value
        PS OFFSET(0) NUMBITS(2) []
    ],

    BackupWakeup [
        BKUP OFFSET(0) NUMBITS(32) [
            Eic =      0b000001,
            Ast =      0b000010,
            Wdt =      0b000100,
            Bod33 =    0b001000,
            Bod18 =    0b010000,
            Picouart = 0b100000
        ]
    ],

    BackupPinMuxing [
        /// Backup Pin Muxing
        BKUPPMUX OFFSET(0) NUMBITS(9) [
            Pb01 = 0b000000001,
            Pa06 = 0b000000010,
            Pa04 = 0b000000100,
            Pa05 = 0b000001000,
            Pa07 = 0b000010000,
            Pc03 = 0b000100000,
            Pc04 = 0b001000000,
            Pc05 = 0b010000000,
            Pc06 = 0b100000000
        ]
    ],

    InputOutputRetention [
        /// Retention on I/O lines after waking up from the BACKUP mode
        RET OFFSET(0) NUMBITS(1) [
            IoLinesNotHeld = 0,
            IoLinesHeld = 1
        ]
    ]
];

const BPM_UNLOCK_KEY: u32 = 0xAA;

const BPM: StaticRef<BpmRegisters> = unsafe { StaticRef::new(0x400F0000 as *const BpmRegisters) };

/// Which power scaling mode the chip should use for internal voltages
///
/// See Tables 42-6 and 42-8 (page 1125) for information of energy usage
/// of different power scaling modes
pub enum PowerScaling {
    /// Mode 0: Default out of reset
    ///
    ///   - Maximum system clock frequency is 32MHz
    ///   - Normal flash speed
    PS0,

    /// Mode 1: Reduced voltage
    ///
    ///   - Maximum system clock frequency is 12MHz
    ///   - Normal flash speed
    ///   - These peripherals are not available in Mode 1:
    ///      - USB
    ///      - DFLL
    ///      - PLL
    ///      - Programming/Erasing Flash
    PS1,

    /// Mode 2:
    ///
    ///   - Maximum system clock frequency is 48MHz
    ///   - High speed flash
    PS2,
}

pub enum CK32Source {
    OSC32K = 0,
    RC32K = 1,
}

#[inline(never)]
pub unsafe fn set_ck32source(source: CK32Source) {
    let control = BPM.pmcon.extract();
    unlock_register(0x1c); // Control
    BPM.pmcon
        .modify_no_read(control, PowerModeControl::CK32S.val(source as u32));
}

unsafe fn unlock_register(register_offset: u32) {
    BPM.unlock
        .write(Unlock::KEY.val(BPM_UNLOCK_KEY) + Unlock::ADDR.val(register_offset));
}

unsafe fn power_scaling_ok() -> bool {
    BPM.sr.is_set(Status::PSOK)
}

// This approach based on `bpm_power_scaling_cpu` from ASF
pub unsafe fn set_power_scaling(ps_value: PowerScaling) {
    // The datasheet says to spin on this before doing anything, ASF
    // doesn't as far as I can tell, but it seems like a good idea
    while !power_scaling_ok() {}

    let control = BPM.pmcon.extract();

    // Unlock PMCON register
    unlock_register(0x1c); // Control

    // Actually change power scaling
    BPM.pmcon.modify_no_read(
        control,
        PowerModeControl::PS.val(ps_value as u32)
            + PowerModeControl::PSCM::WithoutCpuHalt
            + PowerModeControl::PSCREQ::PowerScalingRequested,
    );
}
