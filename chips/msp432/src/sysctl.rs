// System Controller (SYSCTL)

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

const WKEY: u8 = 0x69; // for writing to REBOOT_CTL register
const UNLKEY: u16 = 0x695A; // for unlocking IP protected secure zone
const UNLOCKED: u16 = 0xA596; // value if zone is unlocked

const SYSCTL_BASE: StaticRef<SysCtlRegisters> =
    unsafe { StaticRef::new(0xE004_3000 as *const SysCtlRegisters) };

#[repr(C)]
struct SysCtlRegisters {
    reboot_ctl: ReadWrite<u32, REBOOT_CTL::Register>,
    nmi_ctlstat: ReadWrite<u32, NMI_CTLSTAT::Register>,
    wdtreset_ctl: ReadWrite<u32, WDTRESET_CTL::Register>,
    perihalt_ctl: ReadWrite<u32, PERIHALT_CTL::Register>,
    sram_size: ReadOnly<u32, SRAM_SIZE::Register>,
    sram_banken: ReadWrite<u32, SRAM_BANKEN::Register>,
    sram_bankret: ReadWrite<u32, SRAM_BANKRET::Register>,
    _reserved0: [u32; 1],
    flash_size: ReadOnly<u32, FLASH_SIZE::Register>,
    _reserved1: [u32; 3],
    dio_gltflt_ctl: ReadWrite<u32, DIO_GLTFLT_CTL::Register>,
    _reserved2: [u32; 3],
    secdata_unlock: ReadWrite<u32, SECDATA_UNLOCK::Register>,
    _reserved3: [u32; 1007],
    master_unlock: ReadWrite<u32, MASTER_UNLOCK::Register>,
    bootover_req0: ReadWrite<u32, BOOTOVER_REQ0::Register>,
    bootover_req1: ReadWrite<u32, BOOTOVER_REQ1::Register>,
    bootover_ack: ReadWrite<u32, BOOTOVER_ACK::Register>,
    reset_req: ReadWrite<u32, RESET_REQ::Register>,
    reset_statover: ReadWrite<u32, RESET_STATOVER::Register>,
    system_stat: ReadWrite<u32, SYSTEM_STAT::Register>,
}

register_bitfields! [u32,
    REBOOT_CTL [
        // writing 1 initiates a reboot
        REBOOT OFFSET(0) NUMBITS(1),
        // key to enable writing to bit 0
        WKEY OFFSET(8) NUMBITS(8)
    ],
    // enable/disable interrupts as NMI-sources
    NMI_CTLSTAT [
        // Clock-system
        CS_SRC OFFSET(0) NUMBITS(1),
        // Power Supply System
        PSS_SRC OFFSET(1) NUMBITS(1),
        // Power Control Manager
        PCM_SRC OFFSET(2) NUMBITS(1),
        // Reset Pin
        PIN_SRC OFFSET(3) NUMBITS(1),
        // Status Clock-System
        CS_FLG OFFSET(16) NUMBITS(1),
        // Status Power Supply System
        PSS_FLG OFFSET(17) NUMBITS(1),
        // Status Power Control Manager
        PCM_FLG OFFSET(18) NUMBITS(1),
        // Status Reset Pin
        PIN_FLG OFFSET(19) NUMBITS(1)
    ],
    WDTRESET_CTL [
        // Watchdog timeout: 0 = soft-reset, 1 = hard-reset
        TIMEOUT OFFSET(0) NUMBITS(1),
        // Watchdog password violation: 0 = soft-reset, 1 = hard-reset
        VIOLATION OFFSET(0) NUMBITS(1)
    ],
    // freezes the corresponding peripheral when the CPU is halted
    PERIHALT_CTL [
        // Timer 16bit 0
        T16_0 OFFSET(0) NUMBITS(1),
        // Timer 16bit 1
        T16_1 OFFSET(1) NUMBITS(1),
        // Timer 16bit 2
        T16_2 OFFSET(2) NUMBITS(1),
        // Timer 16bit 3
        T16_3 OFFSET(3) NUMBITS(1),
        // Timer 32bit 0
        T32_0 OFFSET(4) NUMBITS(1),
        EUA0 OFFSET(5) NUMBITS(1),
        EUA1 OFFSET(6) NUMBITS(1),
        EUA2 OFFSET(7) NUMBITS(1),
        EUA3 OFFSET(8) NUMBITS(1),
        EUB0 OFFSET(9) NUMBITS(1),
        EUB1 OFFSET(10) NUMBITS(1),
        EUB2 OFFSET(11) NUMBITS(1),
        EUB3 OFFSET(12) NUMBITS(1),
        ADC OFFSET(13) NUMBITS(1),
        WDT OFFSET(14) NUMBITS(1),
        DMA OFFSET(15) NUMBITS(1)
    ],
    SRAM_SIZE [
        // stores the size of SRAM
        SIZE OFFSET(0) NUMBITS(32)
    ],
    // enable the different SRAM banks, enabling one enables all previous ones
    SRAM_BANKEN [
        BNK0_EN OFFSET(0) NUMBITS(1),
        BNK1_EN OFFSET(1) NUMBITS(1),
        BNK2_EN OFFSET(2) NUMBITS(1),
        BNK3_EN OFFSET(3) NUMBITS(1),
        BNK4_EN OFFSET(4) NUMBITS(1),
        BNK5_EN OFFSET(5) NUMBITS(1),
        BNK6_EN OFFSET(6) NUMBITS(1),
        BNK7_EN OFFSET(7) NUMBITS(1),
        SRAM_RDY OFFSET(16) NUMBITS(1)
    ],
    // choose if the content of the SRAM banks should be retained in LPM3/4
    SRAM_BANKRET [
        BNK0_RET OFFSET(0) NUMBITS(1),
        BNK1_RET OFFSET(1) NUMBITS(1),
        BNK2_RET OFFSET(2) NUMBITS(1),
        BNK3_RET OFFSET(3) NUMBITS(1),
        BNK4_RET OFFSET(4) NUMBITS(1),
        BNK5_RET OFFSET(5) NUMBITS(1),
        BNK6_RET OFFSET(6) NUMBITS(1),
        BNK7_RET OFFSET(7) NUMBITS(1),
        SRAM_RDY OFFSET(16) NUMBITS(1)
    ],
    FLASH_SIZE [
        // stores the size of the flash-memory
        SIZE OFFSET(0) NUMBITS(32)
    ],
    DIO_GLTFLT_CTL [
        // enable/disable glitch-filter for digital IOs
        GLTCH_EN OFFSET(0) NUMBITS(1)
    ],
    SECDATA_UNLOCK [
        // register to unlock IP protected secure zone for data access
        UNLKEY OFFSET(0) NUMBITS(16)
    ],
    MASTER_UNLOCK [
        // unlock SYSTCTL register access from offset 0x1000
        UNLKEY OFFSET(0) NUMBITS(16)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    BOOTOVER_REQ0 [
        VAL OFFSET(0) NUMBITS(32)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    BOOTOVER_REQ1 [
        VAL OFFSET(0) NUMBITS(32)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    BOOTOVER_ACK [
        VAL OFFSET(0) NUMBITS(32)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    RESET_REQ [
        // trigger a power on reset
        POR OFFSET(0) NUMBITS(1),
        // trigger a reboot
        REBOOT OFFSET(1) NUMBITS(1),
        // 0x69 must be written in the same cycle to apply POR or REBOOT
        WKEY OFFSET(8) NUMBITS(8)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    RESET_STATOVER [
        // indicates if soft reset is asserted
        SOFT OFFSET(0) NUMBITS(1),
        // indicates if hard reset is asserted
        HARD OFFSET(1) NUMBITS(1),
        // indicates if reboot reset is asserted
        REBOOT OFFSET(2) NUMBITS(1),
        // override request for the soft reset output of the reset controller
        SOFT_OVER OFFSET(8) NUMBITS(1),
        // override request for the hard reset output of the reset controller
        HARD_OVER OFFSET(9) NUMBITS(1),
        // override request for the reboot reset output of the reset controller
        REBOOT_OVER OFFSET(10) NUMBITS(1)
    ],
    // access allowed only if MASTER_UNLOCK register is unlocked
    SYSTEM_STAT [
        DBG_SEC_ACT OFFSET(3) NUMBITS(1),
        JTAG_SWD_LOCK_ACT OFFSET(4) NUMBITS(1),
        IP_PROT_ACT OFFSET(5) NUMBITS(1)
    ]
];

pub struct SysCtl {
    registers: StaticRef<SysCtlRegisters>,
}

impl SysCtl {
    pub const fn new() -> SysCtl {
        SysCtl {
            registers: SYSCTL_BASE,
        }
    }

    pub fn enable_all_sram_banks(&self) {
        self.registers.sram_banken.modify(SRAM_BANKEN::BNK7_EN::SET);
    }
}
