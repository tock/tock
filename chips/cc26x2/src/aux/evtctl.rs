// Table 19-71. CC26_AUX_EVCTL_MMAP_AUX_EVCTL Registers

// Offset Acronym Register Name Section
// 0h       EVSTAT0             Event Status 0                                             Section 19.8.3.1
// 4h       EVSTAT1             Event Status 1                                             Section 19.8.3.2
// 8h       EVSTAT2             Event Status 2                                             Section 19.8.3.3
// Ch       EVSTAT3             Event Status 3                                             Section 19.8.3.4
// 10h      SCEWEVCFG0          Sensor Controller Engine Wait Event Configuration 0        Section 19.8.3.5
// 14h      SCEWEVCFG1          Sensor Controller Engine Wait Event Configuration 1        Section 19.8.3.6
// 18h      DMACTL              Direct Memory Access Control                               Section 19.8.3.7
// 20h      SWEVSET             Software Event Set                                         Section 19.8.3.8
// 24h      EVTOAONFLAGS        Events To AON Flags                                        Section 19.8.3.9
// 28h      EVTOAONPOL          Events To AON Polarity                                     Section 19.8.3.10
// 2Ch      EVTOAONFLAGSCLR     Events To AON Clear                                        Section 19.8.3.11
// 30h      EVTOMCUFLAGS        Events to MCU Flags                                        Section 19.8.3.12
// 34h      EVTOMCUPOL          Event To MCU Polarity                                      Section 19.8.3.13
// 38h      EVTOMCUFLAGSCLR     Events To MCU Flags Clear                                  Section 19.8.3.14
// 3Ch      COMBEVTOMCUMASK     Combined Event To MCU Mask                                 Section 19.8.3.15
// 40h      EVOBSCFG            Event Observation Configuration                            Section 19.8.3.16
// 44h      PROGDLY             Programmable Delay                                         Section 19.8.3.17
// 48h      MANUAL              Manual                                                     Section 19.8.3.18
// 4Ch      EVSTAT0L            Event Status 0 Low                                         Section 19.8.3.19
// 50h      EVSTAT0H            Event Status 0 High                                        Section 19.8.3.20
// 54h      EVSTAT1L            Event Status 1 Low                                         Section 19.8.3.21
// 58h      EVSTAT1H            Event Status 1 High                                        Section 19.8.3.22
// 5Ch      EVSTAT2L            Event Status 2 Low                                         Section 19.8.3.23
// 60h      EVSTAT2H            Event Status 2 High                                        Section 19.8.3.24
// 64h      EVSTAT3L            Event Status 3 Low                                         Section 19.8.3.25
// 68h      EVSTAT3H            Event Status 3 High                                        Section 19.8.3.26

use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

use memory_map::AUX_EVCTL_BASE;

pub const REG: StaticRef<Registers> = unsafe { StaticRef::new(AUX_EVCTL_BASE as *const Registers) };

#[repr(C)]
pub struct Registers {
    _evt_stat0: ReadOnly<u32>,
    _evt_stat1: ReadOnly<u32>,
    _evt_stat2: ReadOnly<u32>,
    _evt_stat3: ReadOnly<u32>,
    _sce_w_ev_cfg0: ReadOnly<u32>,
    _sce_w_ev_cfg1: ReadOnly<u32>,
    _dma_ctl: ReadOnly<u32>,
    _gap0: ReadOnly<u32>,
    _sw_ev_set: ReadOnly<u32>,
    _ev_to_aon_flags: ReadOnly<u32>,
    _ev_to_aon_pol: ReadOnly<u32>,
    _ev_to_aon_flags_clr: ReadOnly<u32>,
    pub ev_to_mcu_flags: ReadWrite<u32, EvToMcu::Register>,
    _ev_to_mcu_pol: ReadOnly<u32>,
    pub ev_to_mcu_flags_clr: WriteOnly<u32, EvToMcu::Register>,
}

register_bitfields! [
    u32,
    EvToMcu [
        TIMER2_PULSE  OFFSET(15) NUMBITS(1) [],
        TIMER2_EV3    OFFSET(14) NUMBITS(1) [],
        TIMER2_EV2    OFFSET(13) NUMBITS(1) [],
        TIMER2_EV1    OFFSET(12) NUMBITS(1) [],
        TIMER2_EV0    OFFSET(11) NUMBITS(1) [],
        ADC_IRQ                 OFFSET(10) NUMBITS(1) [],
        MCU_OBSMUX0             OFFSET(9) NUMBITS(1) [],
        ADC_FIFO_ALMOST_FULL    OFFSET(8) NUMBITS(1) [],
        ADC_DONE                OFFSET(7) NUMBITS(1) [],
        SMPH_AUTO_TAKE_DONE     OFFSET(6) NUMBITS(1) [],
        TIMER1_EV               OFFSET(5) NUMBITS(1) [],
        TIMER0_EV               OFFSET(4) NUMBITS(1) [],
        TDC_DONE                OFFSET(3) NUMBITS(1) [],
        COMPB OFFSET(2) NUMBITS(1) [],
        COMPA OFFSET(1) NUMBITS(1) [],
        WU_EV OFFSET(0) NUMBITS(1) []
    ]
];
