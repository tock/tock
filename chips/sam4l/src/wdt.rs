//! Implementation of the SAM4L hardware watchdog timer.

use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
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
        KEY OFFSET(24) NUMBITS(8) [],
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
        KEY OFFSET(24) NUMBITS(8) [],
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
const BASE_ADDRESS: *mut WdtRegisters = 0x400F0C00 as *mut WdtRegisters;

pub struct Wdt {
    registers: *mut WdtRegisters,
    enabled: Cell<bool>,
}

pub static mut WDT: Wdt = Wdt::new(BASE_ADDRESS);

impl Wdt {
    const fn new(base_address: *mut WdtRegisters) -> Wdt {
        Wdt {
            registers: base_address,
            enabled: Cell::new(false),
        }
    }

    fn start(&self, period: usize) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        self.enabled.set(true);

        unsafe {
            pm::enable_clock(Clock::PBD(PBDClock::WDT));
        }

        // Choose the best period setting based on what was passed to `start()`
        let scaler = match period {
            0...2 => 7,
            3...6 => 8,
            7...12 => 9,
            13...24 => 10,
            25...48 => 11,
            49...96 => 12,
            97...192 => 13,
            193...384 => 14,
            385...768 => 15,
            769...1536 => 16,
            1537...3072 => 17,
            3073...6144 => 18,
            6145...12288 => 19,
            12289...24576 => 20,
            24577...49152 => 21,
            49153...98304 => 22,
            98305...196608 => 23,
            196609...393216 => 24,
            393217...786432 => 25,
            786433...1572864 => 26,
            1572865...3145728 => 27,
            3145729...6291456 => 28,
            6291457...12582912 => 29,
            12582913...25165824 => 30,
            _ => 31,
        };

        let control1 = Control::CEN::ClockEnable + Control::PSEL.val(scaler)
            + Control::FCD::DoNotRedoCalibration
            + Control::DAR::DisableAfterReset + Control::EN::Enable;
        let control2 = Control::CEN::ClockEnable + Control::PSEL.val(scaler)
            + Control::FCD::DoNotRedoCalibration
            + Control::DAR::DisableAfterReset + Control::EN::Enable;

        // Need to write twice for it to work
        regs.cr.write(Control::KEY.val(0x55) + control1);
        regs.cr.write(Control::KEY.val(0xAA) + control2);
    }

    fn stop(&self) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        // Need to write twice for it to work
        regs.cr.modify(Control::KEY.val(0x55) + Control::EN::CLEAR);
        regs.cr.modify(Control::KEY.val(0xAA) + Control::EN::CLEAR);

        unsafe {
            pm::disable_clock(Clock::PBD(PBDClock::WDT));
        }

        self.enabled.set(false);
    }

    fn tickle(&self) {
        let regs: &WdtRegisters = unsafe { &*self.registers };

        // Need to write the WDTCLR bit twice for it to work
        regs.clr.write(Clear::KEY.val(0x55) + Clear::WDTCLR::SET);
        regs.clr.write(Clear::KEY.val(0xAA) + Clear::WDTCLR::SET);
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
