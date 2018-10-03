//! Always On Module (AON) management
//!
//! AON is a set of peripherals which is _always on_ (eg. the RTC, MCU, etc).
//!
//! The current configuration disables all wake-up selectors, since the
//! MCU never go to sleep and is always active.
//!
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use rtc;

#[repr(C)]
pub struct AonIocRegisters {
    _reserved0: [u32; 3],
    ioc_latch: ReadWrite<u32, IocLatch::Register>,
    ioc_clk32k_ctl: ReadWrite<u32, IocClk::Register>,
}

#[repr(C)]
pub struct AonEventRegisters {
    mcu_wu_sel: ReadWrite<u32>,       // MCU Wake-up selector
    mcu_wu_sel1: ReadWrite<u32>,      // MCU1 Wake-up selector
    event_to_mcu_sel: ReadWrite<u32>, // Event selector for MCU Events
    rtc_sel: ReadWrite<u32>,          // RTC Capture event selector for AON_RTC
}

#[repr(C)]
struct AonPmCtlRegisters {
    _unknown: ReadOnly<u32>,
    aux_clk: ReadWrite<u32, AuxClk::Register>,
    ram_cfg: ReadWrite<u32, RamCfg::Register>,
    _unknown2: [ReadOnly<u8>; 8],
    pwr_ctl: ReadWrite<u32, PwrCtl::Register>,
    pwr_stat: ReadOnly<u32, PwrStat::Register>,
    shutdown: ReadWrite<u32, Shutdown::Register>,
    _recharge_ctl: [u8; 4],
    _recharge_stat: [u8; 4],
    _osc_cfg: ReadWrite<u32, OscCtl::Register>,
    reset_ctl: ReadWrite<u32, ResetCtl::Register>,
    sleep_ctl: ReadWrite<u32, SleepCtl::Register>,
    _jtag_cfg: [ReadOnly<u8>; 4],
    _jtag_usercode: [ReadOnly<u8>; 4],
}

register_bitfields![
    u32,
    AuxClk [
        PWR_DWN_SRC OFFSET(8) NUMBITS(1) [
            NO_CLOCK = 0x00,
            SCLK_LF = 0x01
        ],
        SRC     OFFSET(0) NUMBITS(1) [
            SCLK_HFDIV2 = 0x00,
            SCLK_MF = 0x01
        ]
    ],
    RamCfg [
        AUX_SRAM_PWR_OFF_OFF    OFFSET(17) NUMBITS(1) [],
        AUX_SRAM_RET_EN OFFSET(16) NUMBITS(1) [],

        //  SRAM Retention enabled
        //  0x00 - Retention disabled
        //  0x01 - Retention enabled for BANK0
        //  0x03 - Retention enabled for BANK0, BANK1
        //  0x07 - Retention enabled for BANK0, BANK1, BANK2
        //  0x0F - Retention enabled for BANK0, BANK1, BANK2, BANK3
        BUS_SRAM_RET_EN OFFSET(0)  NUMBITS(4) [
            OFF = 0x00,
            ON = 0x0F   // Default to enable retention in all banks
        ]
    ],
    PwrCtl [
        // 0 = use GLDO in active mode, 1 = use DCDC in active mode
        DCDC_ACTIVE  OFFSET(2) NUMBITS(1) [],
        // 0 = DCDC/GLDO are used, 1 = DCDC/GLDO are bypassed and using a external regulater
        EXT_REG_MODE OFFSET(1) NUMBITS(1) [],
        // 0 = use GDLO for recharge, 1 = use DCDC for recharge
        DCDC_EN      OFFSET(0) NUMBITS(1) []
    ],
    PwrStat [
        JTAG_PD_ON  OFFSET(2) NUMBITS(1) [],
        AUX_BUS_RESET_DONE OFFSET(1) NUMBITS(1) [],
        AUX_RESET_DONE OFFSET(0) NUMBITS(1) []
    ],
    Shutdown [
        // Controls whether MCU & AUX requesting to be powered off
        // will enable a transition to powerdown (0 = Enabled, 1 = Disabled)
        PWR_DWN_DIS     OFFSET(0) NUMBITS(1) []
    ],
    IocClk [
        EN  OFFSET(0) NUMBITS(1) []
    ],
    IocLatch [
        EN OFFSET(0) NUMBITS(1) []
    ],
    OscCtl [
        // Reserved 8-31
        PER_M OFFSET(3) NUMBITS(7) [],
        PER_E OFFSET(0) NUMBITS(3) []
    ],
    ResetCtl [
        // 0: No effect 1: Generate system reset
        SYSRESET OFFSET(31) NUMBITS(1) [],
        // Reserved 26-30
        // 24/25 BOOT DET 0/1 CLR
        // Reserved 18-23
        WU_FROM_SD OFFSET(15) NUMBITS(1) [],
        GPIO_WU_FROM_SD OFFSET(14) NUMBITS(1) [],
        // 12/13 BOOT DET
        // Reserved 9-11
        VDDS_LOSS_EN OFFSET(8) NUMBITS(1) [],
        VDDR_LOSS_EN OFFSET(7) NUMBITS(1) [],
        VDD_LOSS_EN OFFSET(6) NUMBITS(1) [],
        CLK_LOSS_EN OFFSET(5) NUMBITS(1) [],
        MCU_WARM_RESET OFFSET(4) NUMBITS(1) [],
        RESET_SRC OFFSET(1) NUMBITS(3) []
        // Reserved 0
    ],
    SleepCtl [
        IO_PAD_SLEEP_DIS OFFSET(0) NUMBITS(1) []
    ]
];

const AON_EVENT_BASE: StaticRef<AonEventRegisters> =
    unsafe { StaticRef::new(0x4009_3000 as *const AonEventRegisters) };
const AON_PMCTL_BASE: StaticRef<AonPmCtlRegisters> =
    unsafe { StaticRef::new(0x4009_0000 as *const AonPmCtlRegisters) };
const AON_IOC_BASE: StaticRef<AonIocRegisters> =
    unsafe { StaticRef::new(0x4009_4000 as *const AonIocRegisters) };

pub enum AuxSClk {
    SClkHFDiv2,
    SClkMF,
}

pub struct Aon {
    event_regs: StaticRef<AonEventRegisters>,
    pmctl_regs: StaticRef<AonPmCtlRegisters>,
    ioc_regs: StaticRef<AonIocRegisters>,
}

pub const AON: Aon = Aon::new();

impl Aon {
    const fn new() -> Aon {
        Aon {
            event_regs: AON_EVENT_BASE,
            pmctl_regs: AON_PMCTL_BASE,
            ioc_regs: AON_IOC_BASE,
        }
    }

    pub fn setup(&self) {
        let regs = &*self.event_regs;

        // Set RTC CH1 as a wakeup source by default
        regs.mcu_wu_sel.set(0x3F3F3F24);

        // Set RTC CH1 as a wakeup source by default
        regs.mcu_wu_sel1.set(0x3F3F3F24);
        
        // Disable RTC combined event
        regs.rtc_sel.set(0x0000003F);
        
        // The default reset value is 0x002B2B2B. However, 0x2b for each
        // programmable event corresponds to a JTAG event; which is fired
        // *all* the time during debugging through JTAG. It is better to
        // ignore it in this case.
        //      NOTE: the aon programmable interrupt will still be fired
        //            once a debugger is attached through JTAG.
        regs.event_to_mcu_sel.set(0x003F3F3F);
    }

    pub fn set_dcdc_enabled(&self, enabled: bool) {
        let regs = &*self.pmctl_regs;
        if enabled {
            regs.pwr_ctl
                .modify(PwrCtl::DCDC_ACTIVE::SET + PwrCtl::DCDC_EN::SET);
        } else {
            regs.pwr_ctl
                .modify(PwrCtl::DCDC_ACTIVE::CLEAR + PwrCtl::DCDC_EN::CLEAR);
        }
    }

    pub fn lfclk_enable(&self, enable: bool) {
        let regs = &*self.ioc_regs;
        if enable {
            regs.ioc_clk32k_ctl.write(IocClk::EN::SET);
        } else {
            regs.ioc_clk32k_ctl.write(IocClk::EN::CLEAR);
        }
    }
    
    pub fn lock_io_pins(&self, enable: bool) {
        let regs = &*self.ioc_regs;
        if enable {
            regs.ioc_latch.write(IocLatch::EN::CLEAR);
        } else {
            regs.ioc_latch.write(IocLatch::EN::SET);
        }
    }
    
    pub fn aux_set_ram_retention(&self, enabled: bool) {
        let regs = &*self.pmctl_regs;
        regs.ram_cfg.modify({
            if enabled {
                RamCfg::AUX_SRAM_RET_EN::SET
            } else {
                RamCfg::AUX_SRAM_RET_EN::CLEAR
            }
        });
    }

    pub fn mcu_set_ram_retention(&self, on: bool) {
        let regs = &*self.pmctl_regs;
        regs.ram_cfg.modify({
            if on {
                RamCfg::BUS_SRAM_RET_EN::ON
            } else {
                RamCfg::BUS_SRAM_RET_EN::OFF
            }
        });
    }

    pub fn aux_sceclk_select(&self, sclk: AuxSClk) {
        let regs = &*self.pmctl_regs;
        match sclk {
            AuxSClk::SClkHFDiv2 => regs.aux_clk.modify(AuxClk::SRC::SCLK_HFDIV2),
            AuxSClk::SClkMF => regs.aux_clk.modify(AuxClk::SRC::SCLK_MF),
        }
    }

    pub fn aux_set_power_down_clock(&self) {
        let regs = &*self.pmctl_regs;
        regs.aux_clk.modify(AuxClk::PWR_DWN_SRC::SCLK_LF);
    }

    pub fn aux_disable_power_down_clock(&self) {
        let regs = &*self.pmctl_regs;
        regs.aux_clk.modify(AuxClk::PWR_DWN_SRC::NO_CLOCK);
    }
    
    pub fn aux_reset_done(&self) -> bool {
        let regs = &*self.pmctl_regs;
        let aux_reset_done = regs.pwr_stat.is_set(PwrStat::AUX_RESET_DONE);
        if aux_reset_done {
            return true;
        } else {
            return false;
        }
    }

    pub fn aux_bus_reset_done(&self) -> bool {
        let regs = &*self.pmctl_regs;
        let aux_bus_reset_done = regs.pwr_stat.is_set(PwrStat::AUX_BUS_RESET_DONE);
        if aux_bus_reset_done {
            return true;
        } else {
            return false;
        }
    }

    pub fn shutdown(&self) {
        let regs = &*self.pmctl_regs;
        // TODO Must configure and IOC::DIOxx WU_CFG before shutdown enabled
        regs.shutdown.modify(Shutdown::PWR_DWN_DIS::SET);
    }
    /// Await a cycle of the AON domain in order
    /// to sync with it.
    pub fn sync(&self) {
        unsafe { rtc::RTC.sync() };
    }
}
