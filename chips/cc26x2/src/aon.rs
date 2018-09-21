//! Always On Module (AON) management
//!
//! AON is a set of peripherals which is _always on_ (eg. the RTC, MCU, etc).
//!
//! The current configuration disables all wake-up selectors, since the
//! MCU never go to sleep and is always active.
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use rtc;

#[repr(C)]
pub struct AonIocRegisters {
    _reserved0: [u32; 3],
    ioc_clk32k_ctl: ReadWrite<u32, IocClk::Register>,
}

#[repr(C)]
pub struct AonEventRegisters {
    mcu_wu_sel: ReadWrite<u32>, // MCU Wake-up selector
    aux_wu_sel: ReadWrite<u32>, // AUX Wake-up selector
    event_to_mcu_sel: ReadWrite<u32>, // Event selector for MCU Events
    rtc_sel: ReadWrite<u32>, // RTC Capture event selector for AON_RTC
}

#[repr(C)]
struct AonPmCtlRegisters {
    aux_clk: ReadWrite<u32, AuxClk::Register>,
    ram_cfg: ReadWrite<u32, RamCfg::Register>,
    pwr_ctl: ReadWrite<u32, PwrCtl::Register>,
    pwr_stat: ReadOnly<u32, PwrStat::Register>,
    shutdown: ReadWrite<u32, Shutdown::Register>,
    _recharge: [u32; 4],
}

register_bitfields![
    u32,
    AuxClk [
        SRC     OFFSET(0) NUMBITS(1) [
            SCLK_HFDIV2 = 0x00,
            SCLK_MF = 0x01
        ],
        PWR_DWN_SRC OFFSET(8) NUMBITS(1) [
            NO_CLOCK = 0b0,
            SCLK_LF = 0b1
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
    ]

];

const AON_EVENT_BASE: StaticRef<AonEventRegisters> =
    unsafe { StaticRef::new(0x4009_3000 as *const AonEventRegisters) };
const AON_PMCTL_BASE: StaticRef<AonPmCtlRegisters> =
    unsafe { StaticRef::new(0x4009_0000 as *const AonPmCtlRegisters) };
const AON_IOC_BASE: StaticRef<AonIocRegisters> =
    unsafe { StaticRef::new(0x4009_4000 as *const AonIocRegisters) };

pub struct Aon {
    event_regs: StaticRef<AonEventRegisters>,
}

pub const AON: Aon = Aon::new();

impl Aon {
    const fn new() -> Aon {
        Aon { event_regs: AON_EVENT_BASE }
    }

    pub fn setup(&self) {
        let regs = &*self.event_regs;

        // Default to no events at all
        regs.aux_wu_sel.set(0x3F3F3F3F);

        // Set RTC CH1 as a wakeup source by default
        regs.mcu_wu_sel.set(0x3F3F3F24);

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
        let regs = AON_PMCTL_BASE;
        if enabled {
            regs.pwr_ctl.modify(
                PwrCtl::DCDC_ACTIVE::SET + PwrCtl::DCDC_EN::SET,
            );
        } else {
            regs.pwr_ctl.modify(
                PwrCtl::DCDC_ACTIVE::CLEAR +
                    PwrCtl::DCDC_EN::CLEAR,
            );
        }
    }

    pub fn lfclk_enable(&self, enable: bool) {
        let regs = AON_IOC_BASE;
        if enable {
            regs.ioc_clk32k_ctl.write(IocClk::EN::SET);
        } else {
            regs.ioc_clk32k_ctl.write(IocClk::EN::CLEAR);
        }
    }

    pub fn aux_set_ram_retention(&self, enabled: bool) {
        let regs = AON_PMCTL_BASE;
        regs.ram_cfg.modify({
            if enabled {
                RamCfg::AUX_SRAM_RET_EN::SET
            } else {
                RamCfg::AUX_SRAM_RET_EN::CLEAR
            }
        });
    }

    pub fn mcu_set_ram_retention(&self, on: bool) {
        let regs = AON_PMCTL_BASE;
        regs.ram_cfg.modify({
            if on {
                RamCfg::BUS_SRAM_RET_EN::ON
            } else {
                RamCfg::BUS_SRAM_RET_EN::OFF
            }
        });
    }

    pub fn shutdown(&self) {
        let regs = AON_PMCTL_BASE;
        regs.shutdown.modify(Shutdown::PWR_DWN_DIS::SET);
    }
    /// Await a cycle of the AON domain in order
    /// to sync with it.
    pub fn sync(&self) {
        unsafe { rtc::RTC.sync() };
    }
}
