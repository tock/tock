//! Implementation of the stm32f3 watchdog timers.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

const WINDOW_WATCHDOG_BASE: StaticRef<WwdgRegisters> =
    unsafe { StaticRef::new(0x4000_2C00 as *const WwdgRegisters) };

const INDEPENDENT_WATCHDOG_BASE: StaticRef<IwdgRegisters> =
    unsafe { StaticRef::new(0x4000_3000 as *const IwdgRegisters) };

#[repr(C)]
pub struct WwdgRegisters {
    cr: ReadWrite<u32, Control::Register>,
    cfr: ReadWrite<u32, Config::Register>,
    sr: ReadWrite<u32, Status::Register>,
}

#[repr(C)]
pub struct IwdgRegisters {
    kr: WriteOnly<u32, Key::Register>,
    pr: ReadWrite<u32, Prescaler::Register>,
    rlr: ReadWrite<u32, Reload::Register>,
    sr: ReadOnly<u32, IStatus::Register>,
    winr: ReadWrite<u32, Window::Register>,
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

register_bitfields![u32,
    Key [
        /// Key value
        /// These bits must be written by software at regular intervals with
        /// the value 0xAAAA, otherwise the watchdog generates a reset when the
        /// counter reaches 0.
        /// Writing the value 0x5555 to enable access to the Prescaler, Reload
        /// and Window registers.
        /// Writing the value 0xCCCC starts the watchdog (except if the
        /// hardware watchdog option is selected).
        KEY OFFSET(0) NUMBITS(16) []
    ],
    Prescaler [
        /// Prescaler divider
        /// These bits are written by software to select the prescaler divider
        /// feeding the counter clock. PVU bit of the Status register must be
        /// reset in order to be able to change the prescaler divider.
        PR OFFSET(0) NUMBITS(3) [
            /// Divider /4
            DIVIDER4 = 0,
            /// Divider /8
            DIVIDER8 = 1,
            /// Divider /16
            DIVIDER16 = 2,
            /// Divider /32
            DIVIDER32 = 3,
            /// Divider /64
            DIVIDER64 = 4,
            /// Divider /128
            DIVIDER128 = 5,
            /// Divider /256
            DIVIDERA256 = 6,
            /// Divider /256
            DIVIDERB256 = 7
        ]
    ],
    Reload [
        /// Watchdog counter reload value
        /// These bits are written by software to define the value to be loaded
        /// in the watchdog counter each time the value 0xAAAA is written in
        /// Key register. The watchdog counter counts down from this value.
        RL OFFSET(0) NUMBITS(12) []
    ],
    IStatus [
        /// Watchdog counter window value update
        /// This bit is set by hardware to indicate that an update of the
        /// window value is ongoing.
        WVU OFFSET(2) NUMBITS(1) [],
        /// Watchdog counter reload value update
        /// This bit is set by hardware to indicate that an update of the
        /// reload value is ongoing.
        RVU OFFSET(1) NUMBITS(1) [],
        /// Watchdog prescaler value update
        /// This bit is set by hardware to indicate that an update of the
        /// prescaler value is ongoing.
        PVU OFFSET(0) NUMBITS(1) []
    ],
    Window [
        /// Watchdog counter window value
        /// These bits contain the high limit of the window value to be
        /// compared to the downcounter.
        WIN OFFSET(0) NUMBITS(12) []
    ]
];

pub struct WindoWdg {
    registers: StaticRef<WwdgRegisters>,
    client: OptionalCell<&'static dyn kernel::watchdog::WatchdogClient>,
}

impl WindoWdg {
    pub const fn new() -> WindoWdg {
        WindoWdg {
            registers: WINDOW_WATCHDOG_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static dyn kernel::watchdog::WatchdogClient) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.tickle();
        self.client.map(|client| client.reset_happened());
    }

    /// This interrupt is only cleared by hardware after a reset.
    fn enable_interrupt(&self) {
        self.registers.cfr.modify(Config::EWI::SET);
    }

    fn set_window(&self, value: u32) {
        // Set the window value to the biggest possible one.
        self.registers.cfr.modify(Config::W.val(value));
    }

    /// Modifies the time base of the prescaler.
    pub fn set_prescaler(&self, time_base: u8) {
        match time_base {
            1 => self.registers.cfr.modify(Config::WDGTB::DIVONE),
            2 => self.registers.cfr.modify(Config::WDGTB::DIVTWO),
            4 => self.registers.cfr.modify(Config::WDGTB::DIVFOUR),
            8 => self.registers.cfr.modify(Config::WDGTB::DIVEIGHT),
            _ => {}
        }
    }

    pub fn start(&self) {
        self.enable_interrupt();
        self.set_window(0x7F);

        // Set T[6] bit to avoid a reset just when the watchdog is activated.
        self.tickle();
        self.registers.cr.modify(Control::WDGA::SET);
    }

    pub fn tickle(&self) {
        self.registers.cr.modify(Control::T.val(0x7F));
    }
}

impl kernel::watchdog::WatchDog for WindoWdg {
    fn setup(&self) {
        self.start();
    }

    fn tickle(&self) {
        self.tickle();
    }

    fn suspend(&self) {}
    fn resume(&self) {}
}

pub struct IndepWdg {
    registers: StaticRef<IwdgRegisters>,
}

impl IndepWdg {
    pub const fn new() -> IndepWdg {
        IndepWdg {
            registers: INDEPENDENT_WATCHDOG_BASE,
        }
    }

    pub fn set_prescaler(&self, value: u8) {
        // Enable register access
        self.registers.kr.set(0x5555);

        match value {
            0 => self.registers.pr.write(Prescaler::PR::DIVIDER4),
            1 => self.registers.pr.write(Prescaler::PR::DIVIDER8),
            2 => self.registers.pr.write(Prescaler::PR::DIVIDER16),
            3 => self.registers.pr.write(Prescaler::PR::DIVIDER32),
            4 => self.registers.pr.write(Prescaler::PR::DIVIDER64),
            5 => self.registers.pr.write(Prescaler::PR::DIVIDER128),
            6 => self.registers.pr.write(Prescaler::PR::DIVIDERA256),
            7 => self.registers.pr.write(Prescaler::PR::DIVIDERB256),
            _ => {}
        }
    }

    pub fn start(&self) {
        // Activate the Independent watchdog
        self.registers.kr.set(0xCCCC);

        // Enable register access
        self.registers.kr.set(0x5555);

        // Set the value to be loaded in the counter after each tickle
        self.registers.rlr.modify(Reload::RL.val(0xFFF));
        self.tickle();
    }

    pub fn tickle(&self) {
        self.registers.kr.set(0xAAAA);
    }
}

impl kernel::watchdog::WatchDog for IndepWdg {
    fn setup(&self) {
        self.start();
    }

    fn tickle(&self) {
        self.tickle();
    }

    fn suspend(&self) {}
    fn resume(&self) {}
}

pub static mut WATCHDOG: Wdt = Wdt::new();

pub struct Wdt {
    pub windo_wdg: WindoWdg,
    pub indep_wdg: IndepWdg,
    windo_enabled: Cell<bool>,
    indep_enabled: Cell<bool>,
}

impl Wdt {
    pub const fn new() -> Wdt {
        Wdt {
            windo_wdg: WindoWdg::new(),
            indep_wdg: IndepWdg::new(),
            windo_enabled: Cell::new(false),
            indep_enabled: Cell::new(false),
        }
    }

    pub fn enable_windo(&self) {
        self.windo_enabled.set(true);
    }

    pub fn enable_indep(&self) {
        self.indep_enabled.set(true);
    }
}

impl kernel::watchdog::WatchDog for Wdt {
    fn setup(&self) {
        if self.windo_enabled.get() {
            self.windo_wdg.setup();
        }

        if self.indep_enabled.get() {
            self.indep_wdg.setup();
        }
    }

    fn tickle(&self) {
        if self.windo_enabled.get() {
            self.windo_wdg.tickle();
        }

        if self.indep_enabled.get() {
            self.indep_wdg.tickle();
        }
    }

    fn suspend(&self) {}
    fn resume(&self) {}
}
