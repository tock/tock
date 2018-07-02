//! Implementation of the SAM4L hardware watchdog timer.

use core::cell::Cell;
use cortexm4::support;
use kernel::common::math::log_base_two_u64;
use kernel::common::regs::{FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use pm::{self, Clock, PBDClock};

#[repr(C)]
pub struct WdtRegisters {
    cr: ReadWrite<u32, Control::Register>,
    clr: WriteOnly<u32, Clear::Register>,
    sr: ReadOnly<u32, Status::Register>,
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
}

register_bitfields![u32,
    Control [
        /// Write access key
        KEY OFFSET(24) NUMBITS(8) [
            KEY1 = 0x55,
            KEY2 = 0xAA
        ],
        /// Time Ban Prescale Select
        TBAN OFFSET(18) NUMBITS(5) [],
        /// Clock Source Select
        CSSEL OFFSET(17) NUMBITS(1) [
            RCSYS = 0,
            OSC32K = 1
        ],
        /// Clock Enable
        CEN OFFSET(16) NUMBITS(1) [
            ClockDisable = 0,
            ClockEnable = 1
        ],
        /// Time Out Prescale Select
        PSEL OFFSET(8) NUMBITS(5) [],
        /// Flash Calibration Done
        FCD OFFSET(7) NUMBITS(1) [
            RedoCalibration = 0,
            DoNotRedoCalibration = 1
        ],
        /// Interrupt Mode
        IM OFFSET(4) NUMBITS(1) [
            InterruptModeDisabled = 0,
            InterruptModeEnabled = 1
        ],
        /// WDT Control Register Store Final Value
        SFV OFFSET(3) NUMBITS(1) [
            NotLocked = 0,
            Locked = 1
        ],
        /// WDT Mode
        MODE OFFSET(2) NUMBITS(1) [
            Basic = 0,
            Window = 1
        ],
        /// WDT Disable After Reset
        DAR OFFSET(1) NUMBITS(1) [
            EnableAfterReset = 0,
            DisableAfterReset = 1
        ],
        /// WDT Enable
        EN OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    Clear [
        /// Write access key
        KEY OFFSET(24) NUMBITS(8) [
            KEY1 = 0x55,
            KEY2 = 0xAA
        ],
        /// Watchdog Clear
        WDTCLR OFFSET(0) NUMBITS(1) []
    ],

    Status [
        /// WDT Counter Cleared
        CLEARED 1,
        /// Within Window
        WINDOW 0
    ],

    Interrupt [
        WINT 2
    ]
];

// Page 59 of SAM4L data sheet
const WDT_BASE: *mut WdtRegisters = 0x400F0C00 as *mut WdtRegisters;
const WDT_REGS: StaticRef<WdtRegisters> =
    unsafe { StaticRef::new(WDT_BASE as *const WdtRegisters) };

pub struct Wdt {
    enabled: Cell<bool>,
}

pub static mut WDT: Wdt = Wdt::new();

#[derive(Copy, Clone)]
pub enum WdtClockSource {
    ClockRCSys = 0,
    ClockOsc32 = 1,
}

impl From<WdtClockSource> for FieldValue<u32, Control::Register> {
    fn from(clock: WdtClockSource) -> Self {
        match clock {
            WdtClockSource::ClockRCSys => Control::CSSEL::RCSYS,
            WdtClockSource::ClockOsc32 => Control::CSSEL::OSC32K,
        }
    }
}

impl Wdt {
    const fn new() -> Wdt {
        Wdt {
            enabled: Cell::new(false),
        }
    }

    /// WDT Errata: §45.1.3
    ///
    /// When writing any of the PSEL, TBAN, EN, or MODE fields, must insert a
    /// delay for synchronization to complete.
    ///
    /// Also handle the KEY for the caller since we're special casing this.
    fn write_cr(&self, control: FieldValue<u32, Control::Register>) {
        WDT_REGS.cr.modify(Control::KEY::KEY1 + control);
        WDT_REGS.cr.modify(Control::KEY::KEY2 + control);

        // When writing to the affected fields, the user must ensure a wait
        // corresponding to 2 clock cycles of both the WDT peripheral bus clock
        // and the selected WDT clock source.
        //
        // TODO: Actual math based on chosen clock, ASF does:
        //       delay = div_ceil(sysclk_hz(), OSC_[chosen]_NOMINAL_HZ)
        for _ in 0..10000 {
            support::nop();
        }
    }

    fn select_clock(&self, clock: WdtClockSource) {
        if !(WDT_REGS.cr.matches_all(From::from(clock))) {
            let clock_enabled = WDT_REGS.cr.is_set(Control::CEN);

            if clock_enabled {
                // Disable WDT clock before modifying source
                self.write_cr(Control::CEN::CLEAR);
                while WDT_REGS.cr.is_set(Control::CEN) {}
            }

            // Select Clock
            self.write_cr(From::from(clock));

            if clock_enabled {
                // Re-enable WDT clock after modifying source
                self.write_cr(Control::CEN::SET);
                while !WDT_REGS.cr.is_set(Control::CEN) {}
            }
        }
    }

    fn start(&self, period: usize) {
        self.enabled.set(true);

        pm::enable_clock(Clock::PBD(PBDClock::WDT));

        // Note: Must use this clock to allow deep sleep. If you leave the
        // default RCSYS, then the watchdog simply will not fire if you enter
        // deep sleep (despite §20.4.1's protestations to the contrary).
        // This is lower power anyway, so take the win.
        self.select_clock(WdtClockSource::ClockOsc32);

        // Choose the best period setting based on what was passed to `start()`
        //
        // §20.5.1.3 Configuring the WDT
        //
        // T_timeout = T_psel = 2^(PSEL + 1) / f_wdt_clk
        //
        // Period is in ms so use freq in khz for easy integer math
        let f_clk_khz: u64 = if WDT_REGS.cr.matches_all(Control::CSSEL::RCSYS) {
            115
        } else {
            // OSC32K
            32
        };
        let mult: u64 = f_clk_khz * (period as u64);
        let scaler = log_base_two_u64(mult); // prefer rounding for longer WD (thus no -1)

        let control = Control::CEN::ClockEnable
            + Control::PSEL.val(scaler)
            + Control::FCD::DoNotRedoCalibration
            + Control::DAR::DisableAfterReset
            + Control::EN::Enable;
        self.write_cr(control);
    }

    fn stop(&self) {
        self.write_cr(Control::EN::CLEAR);

        pm::disable_clock(Clock::PBD(PBDClock::WDT));

        self.enabled.set(false);
    }

    fn tickle(&self) {
        // Need to write the WDTCLR bit twice for it to work
        WDT_REGS.clr.write(Clear::KEY::KEY1 + Clear::WDTCLR::SET);
        WDT_REGS.clr.write(Clear::KEY::KEY2 + Clear::WDTCLR::SET);
    }
}

impl hil::watchdog::Watchdog for Wdt {
    fn start(&self, period: usize) {
        self.start(period);
    }

    fn stop(&self) {
        self.stop();
    }

    fn tickle(&self) {
        self.tickle();
    }
}
