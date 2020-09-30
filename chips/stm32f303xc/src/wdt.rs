//! Window watchdog timer

use crate::rcc;
use core::cell::Cell;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

const WINDOW_WATCHDOG_BASE: StaticRef<WwdgRegisters> =
    unsafe { StaticRef::new(0x4000_2C00 as *const WwdgRegisters) };

#[repr(C)]
pub struct WwdgRegisters {
    cr: ReadWrite<u32, Control::Register>,
    cfr: ReadWrite<u32, Config::Register>,
    sr: ReadWrite<u32, Status::Register>,
}

register_bitfields![u32,
    Control [
        /// Watch dog activation
        /// Set by software and only cleared by hardware after a reset.
        /// When set, the watchdog can generate a reset.
        WDGA OFFSET(7) NUMBITS(1) [],
        /// 7 bit counter
        /// These bits contain the value of the watchdog counter. It is
        /// decremented every 4096 * 2^WDGTB PCLK cycles. A reset is produced
        /// when it is decremented from 0x40 to 0x3F (T[6] becomes cleared).
        T OFFSET(0) NUMBITS(7) []
    ],
    Config [
        /// Early wakeup interrupt
        /// When set, interrupt occurs whenever the counter reaches the value
        /// of 0x40. This interrupt is only cleared by hardware after a reset.
        EWI OFFSET(9) NUMBITS(1) [],
        /// Timer base
        /// This allows modifying the time base of the prescaler.
        WDGTB OFFSET(7) NUMBITS(2) [
            /// CK Counter Clock (PCLK div 4096) div 1
            DIVONE = 0,
            /// CK Counter Clock (PCLK div 4096) div 2
            DIVTWO = 1,
            /// CK Counter Clock (PCLK div 4096) div 4
            DIVFOUR = 2,
            /// CK Counter Clock (PCLK div 4096) div 8
            DIVEIGHT = 3
        ],
        /// 7 bit window value
        /// These bits contain the window value to be compared to the
        /// downcounter.
        W OFFSET(0) NUMBITS(7) []
    ],
    Status [
        /// Early wakeup interrupt flag
        /// This is set when the counter has reached the value 0x40. It must be
        /// cleared by software by writing 0. This bit is also set when the
        /// interrupt is not enabled.
        EWIF OFFSET(0) NUMBITS(1) []
    ]
];

pub static mut WATCHDOG: WindoWdg = WindoWdg::new();

pub struct WindoWdg {
    registers: StaticRef<WwdgRegisters>,
    clock: WdgClock,
    enabled: Cell<bool>,
}

impl WindoWdg {
    pub const fn new() -> WindoWdg {
        WindoWdg {
            registers: WINDOW_WATCHDOG_BASE,
            clock: WdgClock(rcc::PeripheralClock::APB1(rcc::PCLK1::WWDG)),
            enabled: Cell::new(false),
        }
    }

    pub fn enable(&self) {
        self.enabled.set(true);
    }

    fn set_window(&self, value: u32) {
        // Set the window value to the biggest possible one.
        self.registers.cfr.modify(Config::W.val(value));
    }

    /// Modifies the time base of the prescaler.
    /// 0 - decrements the watchdog every clock cycle
    /// 1 - decrements the watchdog every 2nd clock cycle
    /// 2 - decrements the watchdog every 4th clock cycle
    /// 3 - decrements the watchdog every 8th clock cycle
    fn set_prescaler(&self, time_base: u8) {
        match time_base {
            0 => self.registers.cfr.modify(Config::WDGTB::DIVONE),
            1 => self.registers.cfr.modify(Config::WDGTB::DIVTWO),
            2 => self.registers.cfr.modify(Config::WDGTB::DIVFOUR),
            3 => self.registers.cfr.modify(Config::WDGTB::DIVEIGHT),
            _ => {}
        }
    }

    pub fn start(&self) {
        // Enable the APB1 clock for the watchdog.
        self.clock.enable();

        // This disables the window feature.
        self.set_window(0x7F);
        self.set_prescaler(3);

        // Set T[6] bit to avoid a reset just when the watchdog is activated.
        self.tickle();

        // With the APB1 clock running at 36Mhz we are getting timeout value
        // t_WWDG = (1 / 36000) * 4096 * 2^3 * (63 + 1) = 58ms
        self.registers.cr.modify(Control::WDGA::SET);
    }

    pub fn tickle(&self) {
        self.registers.cr.modify(Control::T.val(0x7F));
    }
}

struct WdgClock(rcc::PeripheralClock);

impl ClockInterface for WdgClock {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

impl kernel::watchdog::WatchDog for WindoWdg {
    fn setup(&self) {
        if self.enabled.get() {
            self.start();
        }
    }

    fn tickle(&self) {
        if self.enabled.get() {
            self.tickle();
        }
    }

    fn suspend(&self) {
        if self.enabled.get() {
            self.clock.disable();
        }
    }

    fn resume(&self) {
        if self.enabled.get() {
            self.clock.enable();
        }
    }
}
