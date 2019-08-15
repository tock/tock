//! Implementation of the Backup System Control Interface (BSCIF) peripheral.

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

#[repr(C)]
struct BscifRegisters {
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
    pclksr: ReadOnly<u32, PowerClocksStatus::Register>,
    unlock: WriteOnly<u32, Unlock::Register>,
    _reserved0: u32,
    oscctrl32: ReadWrite<u32, Oscillator32Control::Register>,
    rc32kcr: ReadWrite<u32, RC32Control::Register>,
    rc32ktune: ReadWrite<u32, RC32kTuning::Register>,
    bod33ctrl: ReadWrite<u32, BodControl::Register>,
    bod33level: ReadWrite<u32, BodLevel::Register>,
    bod33sampling: ReadWrite<u32, BodSamplingControl::Register>,
    bod18ctrl: ReadWrite<u32, BodControl::Register>,
    bot18level: ReadWrite<u32, BodLevel::Register>,
    bod18sampling: ReadWrite<u32, BodSamplingControl::Register>,
    vregcr: ReadWrite<u32, VoltageRegulatorConfig::Register>,
    _reserved1: [u32; 4],
    rc1mcr: ReadWrite<u32, RC1MClockConfig::Register>,
    _reserved2: u32,
    bgctrl: ReadWrite<u32, BandgapControl::Register>,
    bgsr: ReadOnly<u32, BandgapStatus::Register>,
    _reserved3: [u32; 4],
    br0: ReadOnly<u32, Backup::Register>,
    br1: ReadOnly<u32, Backup::Register>,
    br2: ReadOnly<u32, Backup::Register>,
    br3: ReadOnly<u32, Backup::Register>,
}

register_bitfields![u32,
    Interrupt [
        LPBGRDY 12,
        VREGOK 10,
        SSWRDY 9,
        BOD18SYNRDY 8,
        BOD33SYNRDY 7,
        BOD18DET 6,
        BOD33DET 5,
        RC32SAT 4,
        RC32KREFE 3,
        RC32KLOCK 2,
        RC32KRDY 1,
        OSC32RDY 0
    ],

    PowerClocksStatus [
        LPBGRDY 12,
        RC1MRDY 11,
        VREGOK 10,
        SSWRDY 9,
        BOD18SYNRDY 8,
        BOD33SYNRDY 7,
        BOD18DET 6,
        BOD33DET 5,
        RC32SAT 4,
        RC32KREFE 3,
        RC32KLOCK 2,
        RC32KRDY 1,
        OSC32RDY 0
    ],

    Unlock [
        /// Unlock Key
        KEY OFFSET(24) NUMBITS(8) [],
        /// Unlock Address
        ADDR OFFSET(0) NUMBITS(10) []
    ],

    Oscillator32Control [
        /// Oscillator Start-up Time
        STARTUP OFFSET(16) NUMBITS(3) [
            Time0ms = 0,
            Time1ms = 1,
            Time72ms = 2,
            Time143ms = 3,
            Time570ms = 4,
            Time1100ms = 5,
            Time2300ms = 6,
            Time4600ms = 7
        ],
        /// Current Selection
        SELCURR OFFSET(12) NUMBITS(4) [
            CrystalCurrent50nA = 0,
            CrystalCurrent75nA = 1,
            CrystalCurrent100nA = 2,
            CrystalCurrent125nA = 3,
            CrystalCurrent150nA = 4,
            CrystalCurrent175nA = 5,
            CrystalCurrent200nA = 6,
            CrystalCurrent225nA = 7,
            CrystalCurrent250nA = 8,
            CrystalCurrent275nA = 9,
            CrystalCurrent300nA = 10,
            CrystalCurrent325nA = 11,
            CrystalCurrent350nA = 12,
            CrystalCurrent375nA = 13,
            CrystalCurrent400nA = 14,
            CrystalCurrent425nA = 15
        ],
        /// Oscillator Mode
        MODE OFFSET(8) NUMBITS(3) [
            ExternalClock = 0,
            CrystalMode = 1,
            AmplitudeCrystalMode = 3,
            CrystalHighCurrentMode = 4,
            AmplitudeCrystalHighCurrentMode = 5
        ],
        /// 1 KHz output Enable
        EN1K OFFSET(3) NUMBITS(1) [
            OutputDisable = 0,
            OutputEnable = 1
        ],
        /// 32 KHz output Enable
        EN32K OFFSET(2) NUMBITS(1) [
            OutputDisable = 0,
            OutputEnable = 1
        ],
        /// 32 KHz Oscillator Enable
        OSC32EN OFFSET(0) NUMBITS(1) [
            OscillatorDisable = 0,
            OscillatorEnable = 1
        ]
    ],

    RC32Control [
        /// Flash calibration done
        FCD OFFSET(7) NUMBITS(1) [
            ReloadCalib = 0,
            KeepCalib = 1
        ],
        /// Reference select
        REF OFFSET(5) NUMBITS(1) [
            Osc32kReference = 0,
            GclkReference = 1
        ],
        /// Mode Selection
        MODE OFFSET(4) NUMBITS(1) [
            OpenLoop = 0,
            ClosedLoop = 1
        ],
        /// Enable 1 kHz output
        EN1K OFFSET(3) NUMBITS(1) [
            OutputDisable = 0,
            OutputEnable = 1
        ],
        /// Enable 32 KHz output
        EN32K OFFSET(2) NUMBITS(1) [
            OutputDisable = 0,
            OutputEnable = 1
        ],
        /// Temperature Compensation Enable
        TCEN OFFSET(1) NUMBITS(1) [
            NotTempCompensated = 0,
            TempCompensated = 1
        ],
        /// Enable as Generic clock source
        EN OFFSET(0) NUMBITS(1) [
            GclkSourceDisable = 0,
            GclkSourceEnable = 1
        ]
    ],

    RC32kTuning [
        /// Coarse Value
        COARSE OFFSET(16) NUMBITS(7) [],
        /// Fine Value
        FINE OFFSET(0) NUMBITS(6) []
    ],

    BodControl [
        /// BOD Control Register Store Final Value
        SFV OFFSET(31) NUMBITS(1) [
            NotLocked = 0,
            Locked = 1
        ],
        /// BOD Fuse Calibration Done
        FCD OFFSET(30) NUMBITS(1) [
            RedoFlashCalibration = 0,
            DoNotRedoFlashCalibration = 1
        ],
        /// Operation modes
        MODE OFFSET(16) NUMBITS(1) [
            Continuous = 0,
            Sampling = 1
        ],
        /// Action
        ACTION OFFSET(8) NUMBITS(2) [
            No = 0,
            Reset = 1,
            Interrupt = 2
        ],
        /// BOD Hysteresis
        HYST OFFSET(1) NUMBITS(1) [
            No = 0,
            Enabled = 1
        ],
        /// Enable
        EN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    BodSamplingControl [
        /// Prescaler Select
        PSEL OFFSET(8) NUMBITS(4) [],
        /// Clock Source Select
        CSSEL OFFSET(1) NUMBITS(1) [
            Rcsys = 0,
            Ck32k = 1
        ],
        /// Clock Enable
        CEN OFFSET(0) NUMBITS(1) [
            Stop = 0,
            Start = 1
        ]
    ],

    BodLevel [
        /// BOD Threshold Range
        RANGE OFFSET(31) NUMBITS(1) [
            Standard = 0,
            Low = 1
        ],
        /// BOD Value
        VAL OFFSET(0) NUMBITS(6) []
    ],

    VoltageRegulatorConfig [
        /// Store Final Value
        SFV OFFSET(31) NUMBITS(1) [
            ReadWrite = 0,
            ReadOnly = 1
        ],
        /// Stop Switching On Event Enable
        SSWEVT OFFSET(10) NUMBITS(1) [
            NotPeripheralControl = 0,
            PeripheralControl = 1
        ],
        /// Stop Switching
        SSW OFFSET(9) NUMBITS(1) [
            NotStop = 0,
            Stop = 1
        ],
        /// Spread Spectrum Generator Enable
        SSG OFFSET(8) NUMBITS(1) [
            SpreadSpectrumDisable = 0,
            SpreadSpectrumEnable = 1
        ],
        /// Voltage Regulator disable
        DIS OFFSET(0) NUMBITS(1) [
            VoltageRegulatorDisable = 0,
            VoltageRegulatorEnable = 1
        ]
    ],

    RC1MClockConfig [
        /// 1MHz RC Osc Calibration
        CLKCAL OFFSET(8) NUMBITS(5) [],
        /// Flash Calibration Done
        FCD OFFSET(7) NUMBITS(1) [
            RedoFlashCalibration = 0,
            DoNotRedoFlashCalibration = 1
        ],
        /// 1MHz RC Osc Clock Output Enable
        CLKOEN OFFSET(0) NUMBITS(1) [
            NotOutput = 0,
            Output = 1
        ]
    ],

    BandgapControl [
        /// ADC Input Selection
        ADCISEL OFFSET(0) NUMBITS(2) [
            NoConnection = 0,
            ADCVoltageReference = 2
        ]
    ],

    BandgapStatus [
        /// Voltage Reference Used by the System
        VREF OFFSET(18) NUMBITS(2) [
            BothUsed = 0,
            BandgapUsed = 1,
            LowPowerBandgapUsed = 2,
            NeitherUsed = 3
        ],
        /// Low Power Bandgap Voltage Reference Ready
        LPBGRDY OFFSET(17) NUMBITS(1) [
            NotReady = 0,
            Ready = 1
        ],
        /// Bandgap Voltage Reference Ready
        BGRDY OFFSET(16) NUMBITS(1) [
            NotReady = 0,
            Ready = 1
        ],
        /// Bandgap Buffer Ready
        BGBUFRDY OFFSET(0) NUMBITS(8) [
            NotReady = 0,
            Ready = 1
        ]
    ],

    Backup [
        DATA OFFSET(0) NUMBITS(32) []
    ]
];

const BSCIF: StaticRef<BscifRegisters> =
    unsafe { StaticRef::new(0x400F0400 as *const BscifRegisters) };

/// Setup the internal 32kHz RC oscillator.
pub fn enable_rc32k() {
    let rc32kcr = BSCIF.rc32kcr.extract();
    // Unlock the BSCIF::RC32KCR register
    BSCIF
        .unlock
        .write(Unlock::KEY.val(0xAA) + Unlock::ADDR.val(0x24));
    // Write the BSCIF::RC32KCR register.
    // Enable the generic clock source, the temperature compensation, and the
    // 32k output.
    BSCIF.rc32kcr.modify_no_read(
        rc32kcr,
        RC32Control::EN32K::OutputEnable
            + RC32Control::TCEN::TempCompensated
            + RC32Control::EN::GclkSourceEnable,
    );
    // Wait for it to be ready, although it feels like this won't do anything
    while !BSCIF.rc32kcr.is_set(RC32Control::EN) {}

    // Load magic calibration value for the 32KHz RC oscillator
    //
    // Unlock the BSCIF::RC32KTUNE register
    BSCIF
        .unlock
        .write(Unlock::KEY.val(0xAA) + Unlock::ADDR.val(0x28));
    // Write the BSCIF::RC32KTUNE register
    BSCIF
        .rc32ktune
        .write(RC32kTuning::COARSE.val(0x1d) + RC32kTuning::FINE.val(0x15));
}

pub fn rc32k_enabled() -> bool {
    BSCIF.rc32kcr.is_set(RC32Control::EN)
}

pub fn setup_rc_1mhz() {
    let rc1mcr = BSCIF.rc1mcr.extract();
    // Unlock the BSCIF::RC32KCR register
    BSCIF
        .unlock
        .write(Unlock::KEY.val(0xAA) + Unlock::ADDR.val(0x58));
    // Enable the RC1M
    BSCIF
        .rc1mcr
        .modify_no_read(rc1mcr, RC1MClockConfig::CLKOEN::Output);

    // Wait for the RC1M to be enabled
    while !BSCIF.rc1mcr.is_set(RC1MClockConfig::CLKOEN) {}
}

pub unsafe fn disable_rc_1mhz() {
    let rc1mcr = BSCIF.rc1mcr.extract();
    // Unlock the BSCIF::RC32KCR register
    BSCIF
        .unlock
        .write(Unlock::KEY.val(0xAA) + Unlock::ADDR.val(0x58));
    // Disable the RC1M
    BSCIF
        .rc1mcr
        .modify_no_read(rc1mcr, RC1MClockConfig::CLKOEN::NotOutput);

    // Wait for the RC1M to be disabled
    while BSCIF.rc1mcr.is_set(RC1MClockConfig::CLKOEN) {}
}
