//! Low Power Management driver.

use kernel::common::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::watchdog::WatchDog;

pub const RTC_CNTL_BASE: StaticRef<RtcCntlRegisters> =
    unsafe { StaticRef::new(0x6000_8000 as *const RtcCntlRegisters) };

register_structs! {
    pub RtcCntlRegisters {
        (0x000 => options0: ReadWrite<u32>),
        (0x004 => slp_timer0: ReadWrite<u32>),
        (0x008 => slp_timer1: ReadWrite<u32>),
        (0x00C => time_updaet: ReadWrite<u32>),
        (0x010 => time_low0: ReadWrite<u32>),
        (0x014 => time_high0: ReadWrite<u32>),
        (0x018 => state0: ReadWrite<u32>),
        (0x01C => timer1: ReadWrite<u32>),
        (0x020 => timer2: ReadWrite<u32>),
        (0x024 => timer3: ReadWrite<u32>),
        (0x028 => timer4: ReadWrite<u32>),
        (0x02C => timer5: ReadWrite<u32>),
        (0x030 => timer6: ReadWrite<u32>),
        (0x034 => ana_conf: ReadWrite<u32>),
        (0x038 => reset_state: ReadWrite<u32>),
        (0x03C => wakeup_state: ReadWrite<u32>),
        (0x040 => int_ena: ReadWrite<u32>),
        (0x044 => int_raw: ReadWrite<u32>),
        (0x048 => int_st: ReadWrite<u32>),
        (0x04C => int_clr: ReadWrite<u32>),
        (0x050 => store0: ReadWrite<u32>),
        (0x054 => store1: ReadWrite<u32>),
        (0x058 => store2: ReadWrite<u32>),
        (0x05C => store3: ReadWrite<u32>),
        (0x060 => ext_xtl_conf: ReadWrite<u32>),
        (0x064 => ext_wakeup_conf: ReadWrite<u32>),
        (0x068 => slp_reject_conf: ReadWrite<u32>),
        (0x06C => cpu_period_conf: ReadWrite<u32>),
        (0x070 => clk_conf: ReadWrite<u32>),
        (0x074 => slow_clk_conf: ReadWrite<u32>),
        (0x078 => sdio_conf: ReadWrite<u32>),
        (0x07C => bias_conf: ReadWrite<u32>),
        (0x080 => vreg: ReadWrite<u32>),
        (0x084 => pwc: ReadWrite<u32>),
        (0x088 => dig_pwc: ReadWrite<u32>),
        (0x08C => dig_iso: ReadWrite<u32>),
        (0x090 => wdtconfig0: ReadWrite<u32, WDTCONFIG0::Register>),
        (0x094 => wdtconfig1: ReadWrite<u32>),
        (0x098 => wdtconfig2: ReadWrite<u32>),
        (0x09C => wdtconfig3: ReadWrite<u32>),
        (0x0A0 => wdtconfig4: ReadWrite<u32>),
        (0x0A4 => wdtfeed: ReadWrite<u32>),
        (0x0A8 => wdtprotect: ReadWrite<u32>),
        (0x0AC => swd_conf: ReadWrite<u32, SWD_CONF::Register>),
        (0x0B0 => swd_wprotect: ReadWrite<u32>),
        (0x0B4 => sw_cpu_stall: ReadWrite<u32>),
        (0x0B8 => store4: ReadWrite<u32>),
        (0x0BC => store5: ReadWrite<u32>),
        (0x0C0 => store6: ReadWrite<u32>),
        (0x0C4 => store7: ReadWrite<u32>),
        (0x0C8 => low_power_st: ReadWrite<u32>),
        (0x0CC => daig0: ReadWrite<u32>),
        (0x0D0 => pad_hold: ReadWrite<u32>),

        (0x0D4 => _reserved0),
        (0x10C => fib_sel: ReadWrite<u32, FIB_SEL::Register>),
        (0x110 => @END),
    }
}

register_bitfields![u32,
    WDTCONFIG0 [
        CHIP_RESET_EN OFFSET(8) NUMBITS(1) [],
        PAUSE_INSLEEP OFFSET(9) NUMBITS(1) [],
        APPCPU_RESET_EN OFFSET(10) NUMBITS(1) [],
        PROCPU_RESET_EN OFFSET(11) NUMBITS(1) [],
        FLASHBOOT_MOD_EN OFFSET(12) NUMBITS(1) [],
        SYS_RESET_LENGTH OFFSET(13) NUMBITS(3) [],
        CPU_RESET_LENGTH OFFSET(16) NUMBITS(3) [],
        STG3 OFFSET(19) NUMBITS(3) [],
        STG2 OFFSET(22) NUMBITS(3) [],
        STG1 OFFSET(25) NUMBITS(3) [],
        STG0 OFFSET(28) NUMBITS(3) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    SWD_CONF [
        AUTO_FEED OFFSET(31) NUMBITS(1) [],
    ],
    FIB_SEL [
        FIB_SEL OFFSET(0) NUMBITS(3) [
            GLITCH_RST = 1,
            BOR_RST = 2,
            SUPER_WDT_RST = 3,
        ],
    ],
];

pub struct RtcCntl {
    registers: StaticRef<RtcCntlRegisters>,
}

impl<'a> RtcCntl {
    pub const fn new(base: StaticRef<RtcCntlRegisters>) -> RtcCntl {
        Self { registers: base }
    }

    /// Enable WDT config writes
    fn enable_wdt_access(&self) {
        self.registers.wdtprotect.set(0x50d8_3aa1);
    }

    /// Disable WDT config writes
    fn disable_wdt_access(&self) {
        self.registers.wdtprotect.set(0);
    }

    pub fn disable_wdt(&self) {
        self.enable_wdt_access();

        self.registers
            .wdtconfig0
            .modify(WDTCONFIG0::EN::CLEAR + WDTCONFIG0::FLASHBOOT_MOD_EN::CLEAR);
        if self
            .registers
            .wdtconfig0
            .is_set(WDTCONFIG0::FLASHBOOT_MOD_EN)
        {
            panic!("Can't disable RTC CNTL WDT");
        }

        self.disable_wdt_access();
    }

    /// Enable SW WDT config writes
    fn enable_sw_wdt_access(&self) {
        self.registers.swd_wprotect.set(0x8F1D_312A);
    }

    /// Disable SW WDT config writes
    fn disable_sw_wdt_access(&self) {
        self.registers.swd_wprotect.set(0);
    }

    pub fn disable_super_wdt(&self) {
        self.registers.fib_sel.modify(FIB_SEL::FIB_SEL::BOR_RST);

        self.enable_sw_wdt_access();
        self.registers.swd_conf.modify(SWD_CONF::AUTO_FEED::SET);
        self.disable_sw_wdt_access();
    }
}

impl WatchDog for RtcCntl {
    fn setup(&self) {}

    fn tickle(&self) {}

    fn suspend(&self) {}

    fn resume(&self) {
        self.tickle();
    }
}
