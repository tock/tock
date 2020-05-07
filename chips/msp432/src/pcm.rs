use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

const PCMKEY: u16 = 0x695A; // for unlocking PCMCTL0 and PCMCTL1

#[repr(C)]
struct PcmRegisters {
    ctl0: ReadWrite<u32, PCMCTL0::Register>,
    ctl1: ReadWrite<u32, PCMCTL1::Register>,
    ie: ReadWrite<u32, PCMIE::Register>,
    ifg: ReadOnly<u32, PCMIFG::Register>,
    clr_ifg: WriteOnly<u32, PCMCLRIFG::Register>,
}

register_bitfields![u32,
    PCMCTL0 [
        // select an active mode
        AMR OFFSET(0) NUMBITS(4),
        // select a low-power mode
        LPMR OFFSET(4) NUMBITS(4),
        // read current power mode
        CPM OFFSET(8) NUMBITS(6),
        // for changing AMR or CPM 0x695A has to be written to this field
        PCMKEY OFFSET(16) NUMBITS(16)
    ],
    PCMCTL1 [
        LOCKLPM5 OFFSET(0) NUMBITS(1),
        LOCKBKUP OFFSET(1) NUMBITS(1),
        FORCE_LPM_ENTRY OFFSET(2) NUMBITS(1),
        PMR_BUSY OFFSET(8) NUMBITS(1),
        PCMKEY OFFSET(16) NUMBITS(16)
    ],
    // interrupt enable register
    PCMIE [
        // invalid transition from active mode to a low-power mode
        LPM_INVALID_TR_IE OFFSET(0) NUMBITS(1),
        // invalid clock setting during a LPM3/LPMx.5 transition
        LPM_INVALID_CLK_IE OFFSET(1) NUMBITS(1),
        // invalid transition setting during a active power mode request
        AM_INVALID_TR_IE OFFSET(2) NUMBITS(1),
        // 'a DC-DC operation cannot be achieved or maintained'
        DCDC_ERROR_IE OFFSET(6) NUMBITS(1)
    ],
    // interrupt flag register
    PCMIFG [
        // invalid transition from active mode to a low-power mode
        LPM_INVALID_TR_IFG OFFSET(0) NUMBITS(1),
        // invalid clock setting during a LPM3/LPMx.5 transition
        LPM_INVALID_CLK_IFG OFFSET(1) NUMBITS(1),
        // invalid transition setting during a active power mode request
        AM_INVALID_TR_IFG OFFSET(2) NUMBITS(1),
        // 'a DC-DC operation cannot be achieved or maintained'
        DCDC_ERROR_IFG OFFSET(6) NUMBITS(1)
    ],
    // interrupt clear register
    PCMCLRIFG [
        // invalid transition from active mode to a low-power mode
        LPM_INVALID_TR_IFG OFFSET(0) NUMBITS(1),
        // invalid clock setting during a LPM3/LPMx.5 transition
        LPM_INVALID_CLK_IFG OFFSET(1) NUMBITS(1),
        // invalid transition setting during a active power mode request
        AM_INVALID_TR_IFG OFFSET(2) NUMBITS(1),
        // 'a DC-DC operation cannot be achieved or maintained'
        DCDC_ERROR_IFG OFFSET(6) NUMBITS(1)
    ]
];
