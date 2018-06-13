//! Implementation of the power manager (PM) peripheral.

use bpm;
use bscif;
use core::cell::Cell;
use core::sync::atomic::Ordering;
use flashcalw;
use gpio;
use kernel::common::regs::{FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;
use scif;

/// §10.7 PM::UserInterface from SAM4L Datasheet.
#[repr(C)]
struct PmRegisters {
    mcctrl: ReadWrite<u32, MainClockControl::Register>,
    cpusel: ReadWrite<u32, CpuClockSelect::Register>,
    _reserved1: u32,
    pbasel: ReadWrite<u32, PeripheralBusXClockSelect::Register>,
    pbbsel: ReadWrite<u32, PeripheralBusXClockSelect::Register>,
    pbcsel: ReadWrite<u32, PeripheralBusXClockSelect::Register>,
    pbdsel: ReadWrite<u32, PeripheralBusXClockSelect::Register>,
    _reserved2: u32,
    cpumask: ReadWrite<u32, ClockMaskCpu::Register>, // 0x020
    hsbmask: ReadWrite<u32, ClockMaskHsb::Register>,
    pbamask: ReadWrite<u32, ClockMaskPba::Register>,
    pbbmask: ReadWrite<u32, ClockMaskPbb::Register>,
    pbcmask: ReadWrite<u32, ClockMaskPbc::Register>,
    pbdmask: ReadWrite<u32, ClockMaskPbd::Register>,
    _reserved3: [u32; 2],
    pbadivmask: ReadWrite<u32, DividedClockMask::Register>, // 0x040
    _reserved4: [u32; 4],
    cfdctrl: ReadWrite<u32, ClockFailureDetectorControl::Register>,
    unlock: WriteOnly<u32, PmUnlock::Register>,
    _reserved5: [u32; 25],                            // 0x60
    ier: WriteOnly<u32, InterruptOrStatus::Register>, // 0xC0
    idr: WriteOnly<u32, InterruptOrStatus::Register>,
    imr: ReadOnly<u32, InterruptOrStatus::Register>,
    isr: ReadOnly<u32, InterruptOrStatus::Register>,
    icr: WriteOnly<u32, InterruptOrStatus::Register>,
    sr: ReadOnly<u32, InterruptOrStatus::Register>,
    _reserved6: [u32; 34],                                  // 0x100
    ppcr: ReadWrite<u32, PeripheralPowerControl::Register>, // 0x160
    _reserved7: [u32; 7],
    rcause: ReadOnly<u32, ResetCause::Register>, // 0x180
    wcause: ReadOnly<u32, WakeCause::Register>,
    awen: ReadWrite<u32, AsynchronousWakeUpEnable::Register>,
    _protctrl: u32, // This register is named, but undocumented in the datasheet
    _reserved8: u32,
    fastsleep: ReadWrite<u32, FastSleep::Register>,
    _reserved9: [u32; 152],
    config: ReadOnly<u32, Configuration::Register>, // 0x200
    version: ReadOnly<u32, Version::Register>,
}

register_bitfields![u32,
    MainClockControl [
        /// Main Clock Selection:
        ///
        /// 0: RCSYS
        /// 1: OSC0
        /// 2: PLL
        /// 3: DFLL
        /// 4: RC80M (requires dividing down before use!)
        /// 5: RCFAST
        /// 6: RC1M
        /// 7: Reserved
        MCSEL OFFSET(0) NUMBITS(3) []
    ],

    /// Note: Writing this register clears SR.CKRDY. Must not write again until SR.CKRDY high.
    CpuClockSelect [
        /// CPU Division: Set to 1 to divide main clock by 2^(CPUSEL+1).
        CPUDIV OFFSET(7) NUMBITS(1) [],

        /// Exponent for CPU clock divider. Must be 0 if CPUDIV is 0.
        CPUSEL OFFSET(0) NUMBITS(3) []
    ],

    /// Note: Writing this register clears SR.CKRDY. Must not write again until SR.CKRDY high.
    PeripheralBusXClockSelect [
        /// APBx Divisor: Set to 1 to divide APBx clock by 2^(PBSEL+1).
        PBDIV OFFSET(7) NUMBITS(1) [],

        /// Exponent for APBx clock divider. Must be 0 if PBDIV is 0.
        PBSEL OFFSET(0) NUMBITS(3) []
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x1, OCD enabled.
    ClockMaskCpu [
        OCD 0
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x1e2, FLASHCALW, APB[A-D] bridge enabled.
    ClockMaskHsb [
        PDCA 0,
        FLASHCALW 1,
        FLASHCALW_PICOCACHE 2,
        USBC 3,
        CRCCU 4,
        APBA_BRIDGE 5,
        APBB_BRIDGE 6,
        APBC_BRIDGE 7,
        APBD_BRIDGE 8,
        AESA 9
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x0, all disabled.
    ClockMaskPba [
        IISC 0,
        SPI 1,
        TC0 2,
        TC1 3,
        TWIM0 4,
        TWIS0 5,
        TWIM1 6,
        TWIS1 7,
        USART0 8,
        USART1 9,
        USART2 10,
        USART3 11,
        ADCIFE 12,
        DACC 13,
        ACIFC 14,
        GLOC 15,
        ABDACB 16,
        TRNG 17,
        PARC 18,
        CATB 19,
        TWIM2 21,
        TWIM3 22,
        LCDCA 23
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x1, FLASHCALW enabled.
    ClockMaskPbb [
        FLASHCALW 0,
        HRAMC1 1,
        HMATRIX 2,
        PDCA 3,
        CRCCU 4,
        USBC 5,
        PEVC 6
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x1f, PM, CHIPID, SCIF, FREQM, and GPIO enabled.
    ClockMaskPbc [
        PM 0,
        CHIPID 1,
        SCIF 2,
        FREQM 3,
        GPIO 4
    ],

    /// If bit n is cleared, the clock for module n is stopped. If bit n is set
    /// the clock for module n is enabled according to the current / power mode.
    ///
    /// Reset Default: 0x3f, BPM, BSCIF, AST, WDT, EIC, PICOUART enabled.
    ClockMaskPbd [
        BPM 0,
        BSCIF 1,
        AST 2,
        WDT 3,
        EIC 4,
        PICOUART 5
    ],

    /// If bit n is written to zero, the clock divided by 2^(n+1) is stopped.
    /// If bit n is written to one, the clock divided by 2^(n+1) is enabled
    /// according to the current power mode.
    ///
    /// Reset Default: 0x7f, all enabled.
    DividedClockMask [
        MASK OFFSET(0) NUMBITS(7) [
            /// TC0 and TC1
            TIMER_CLOCK2 = 1 << 0,

            /// TC0 and TC1, and USART0-3
            TIMER_CLOCK3 = 1 << 2,

            /// TC0 and TC1
            TIMER_CLOCK4 = 1 << 4,

            /// TC0 and TC1
            TIMER_CLOCK5 = 1 << 6
        ]
    ],

    ClockFailureDetectorControl [
        /// Store final value. If set to 1, register becomes read-only.
        SFV 31,

        /// Clock failure detector enable
        CFDEN 0
    ],

    PmUnlock [
        /// Write 0xAA to enable unlock
        KEY OFFSET(24) NUMBITS(8) [],

        /// Register address to unlock. Next APB access must write register specified here.
        ADDR OFFSET(0) NUMBITS(10) []
    ],

    InterruptOrStatus [
        /// Access Error: (1) Write to a protect register without an unlock
        AE 31,

        /// Wakeup Event Occurred: (1) Check WCAUSE register for wakeup source
        WAKE 8,

        /// Clock Ready: Synchronous clocks (0) written but not settled, (1) are ready
        CKRDY 5,

        /// Clock Failure Detected: (0) running correctly, (1) failure detected, reverting to RCSYS
        CFD 0
    ],

    /// Reset Value: 0x1fe
    PeripheralPowerControl [
        /// On powerup, (0) flash waits for BOD18 to be ready, (1) does not wait
        FWBOD18 10,

        /// When waking up, (0) flash waits for bandgap to be ready, (1) does not wait
        FWBGREF 9,

        /// VREG Request Clock Mask, (0) disabled, (1) enabled
        VREGRCMASK 8,

        /// ADCIFE Request Clock Mask, (0) disabled, (1) enabled
        ADCIFERCMASK 7,

        /// PEVC Request Clock Mask, (0) disabled, (1) enabled
        PEVCRCMASK 6,

        /// TWIS1 Request Clock Mask, (0) disabled, (1) enabled
        TWIS1RCMASK 5,

        /// TWIS0 Request Clock Mask, (0) disabled, (1) enabled
        TWIS0RCMASK 4,

        /// AST Request Clock Mask, (0) disabled, (1) enabled
        ASTRCMASK 3,

        /// ACIFC Request Clock Mask, (0) disabled, (1) enabled
        ACIFCRCMASK 2,

        /// CAT Request Clock Mask, (0) disabled, (1) enabled
        CATBRCMASK 1,

        /// Reset Pullup, pullup on external reset pin is (0) disabled, (1) enabled
        RSTPUN 0
    ],

    ResetCause [
        /// Brown-out 3.3V reset (supply voltage too low)
        BOD33 13,

        /// Power-on reset (I/O voltage too low)
        POR33 10,

        /// OCD Reset (the SYSRESETREQ bit in AIRCR of the CPU was written to 1)
        OCDRST 8,

        /// Backup reset
        BKUP 6,

        /// Watchdog reset
        WDT 3,

        /// External reset pin (RESET_N was pulled low)
        EXT 2,

        /// Brown-out reset (core voltage below brown-out threshold)
        BOD 1,

        /// Power-on reset (core voltage below power-on threshold)
        POR 0
    ],

    WakeCause [
        AST 17,
        EIC 16,
        LCDCA 7,
        PICOUART 6,
        BOD33_IRQ 5,
        BOD18_IRQ 4,
        PSOK 3,
        USBC 2,
        TWIS1 1,
        TWIS0 0
    ],

    /// For each bit, if set, the wakeup is enabled
    AsynchronousWakeUpEnable [
        LCDCA 7,
        PICOUART 6,
        BOD33_IRQ 5,
        BOD18_IRQ 4,
        PSOK 3,
        USBC 2,
        TWIS1 1,
        TWIS0 0
    ],

    /// Each bit in this register corresponds to a clock source set as the main
    /// clock just before entering power save mode and just after wake-up to
    /// make the wakeup time faster.
    ///
    /// 0: The corresponding clock source is not set as the main clock after wake-up.
    /// 1: The corresponding clock source is set as the main clock after wake-up.
    FastSleep [
        DFLL 24,
        RC1M 18,
        RCFAST 17,
        RC80 16,
        PLL 8,
        OSC 0
    ],

    Configuration [
        /// HSB PEVC clock implemented
        HSBPEVC 7,

        /// APBD implemented
        PBD 3,

        /// APBC implemented
        PBC 2,

        /// APBB implemented
        PBB 1,

        /// APBA implemented
        PBA 0
    ],

    Version [
        /// Reserved. No functionality associated.
        VARIANT OFFSET(16) NUMBITS(4) [],

        VERSION OFFSET(0) NUMBITS(12) []
    ]
];

pub enum MainClock {
    RCSYS,
    OSC0,
    PLL,
    DFLL,
    RC80M,
    RCFAST,
    RC1M,
}

#[derive(Copy, Clone, Debug)]
pub enum Clock {
    HSB(HSBClock),
    PBA(PBAClock),
    PBB(PBBClock),
    PBC(PBCClock),
    PBD(PBDClock),
}

#[derive(Copy, Clone, Debug)]
pub enum HSBClock {
    PDCA,
    FLASHCALW,
    FLASHCALWP,
    USBC,
    CRCCU,
    APBA,
    APBB,
    APBC,
    APBD,
    AESA,
}

#[derive(Copy, Clone, Debug)]
pub enum PBAClock {
    IISC,
    SPI,
    TC0,
    TC1,
    TWIM0,
    TWIS0,
    TWIM1,
    TWIS1,
    USART0,
    USART1,
    USART2,
    USART3,
    ADCIFE,
    DACC,
    ACIFC,
    GLOC,
    ABSACB,
    TRNG,
    PARC,
    CATB,
    NULL,
    TWIM2,
    TWIM3,
    LCDCA,
}

#[derive(Copy, Clone, Debug)]
pub enum PBBClock {
    FLASHCALW,
    HRAMC1,
    HMATRIX,
    PDCA,
    CRCCU,
    USBC,
    PEVC,
}

#[derive(Copy, Clone, Debug)]
pub enum PBCClock {
    PM,
    CHIPID,
    SCIF,
    FREQM,
    GPIO,
}

#[derive(Copy, Clone, Debug)]
pub enum PBDClock {
    BPM,
    BSCIF,
    AST,
    WDT,
    EIC,
    PICOUART,
}

/// Frequency of the external oscillator. For the SAM4L, different
/// configurations are needed for different ranges of oscillator frequency, so
/// based on the input frequency, various configurations may need to change.
/// When additional oscillator frequencies are needed, they should be added
/// here and the `setup_system_clock` function should be modified to support
/// it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OscillatorFrequency {
    /// 16 MHz external oscillator
    Frequency16MHz,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RcfastFrequency {
    Frequency4MHz,
    Frequency8MHz,
    Frequency12MHz,
}

/// Configuration for the startup time of the external oscillator. In practice
/// we have found that some boards work with a short startup time, while others
/// need a slow start in order to properly wake from sleep. In general, we find
/// that for systems that do not work, at fast speed, they will hang or panic
/// after several entries into WAIT mode.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OscillatorStartup {
    /// Use a fast startup. ~0.5 ms in practice.
    FastStart,

    /// Use a slow startup. ~8.9 ms in practice.
    SlowStart,
}

/// Which source the system clock should be generated from. These are specified
/// as system clock source appended with the clock that it is sourced from
/// appended with the final frequency of the system. So for example, one option
/// is to use the DFLL sourced from the RC32K with a final frequency of 48 MHz.
///
/// When new options (either sources or final frequencies) are needed, they
/// should be added to this list, and then the `setup_system_clock` function
/// can be modified to support it. This is necessary because configurations
/// must be changed not just with the input source but also based on the
/// desired final frequency.
///
/// For options utilizing an external oscillator, the configurations for that
/// oscillator must also be provided.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SystemClockSource {
    /// Use the RCSYS clock (which the system starts up on anyways). Final
    /// system frequency will be 115 kHz. Note that while this is the default,
    /// Tock is NOT guaranteed to work on this setting and will likely fail.
    RcsysAt115kHz,

    RC1M,

    RCFAST {
        frequency: RcfastFrequency,
    },

    /// Use an external crystal oscillator as the direct source for the
    /// system clock. The final system frequency will match the frequency of
    /// the external oscillator.
    ExternalOscillator {
        frequency: OscillatorFrequency,
        startup_mode: OscillatorStartup,
    },

    /// Use an external crystal oscillator as the input to the internal phase
    /// locked loop (PLL) for the system clock. This results in a final
    /// frequency of 48 MHz.
    PllExternalOscillatorAt48MHz {
        frequency: OscillatorFrequency,
        startup_mode: OscillatorStartup,
    },

    /// Use the internal digital frequency locked loop (DFLL) sourced from
    /// the internal RC32K clock. Note this typically requires calibration
    /// of the RC32K to have a consistent clock. Final frequency of 48 MHz.
    DfllRc32kAt48MHz,

    RC80M,
}

pub enum ClockMask {
    RCSYS = 0x01,
    RC1M = 0x02,
    RCFAST = 0x04,
    OSC0 = 0x08,
    DFLL = 0x10,
    PLL = 0x20,
    RC80M = 0x40,
}

const HSB_MASK_OFFSET: u32 = 0x24;
const PBA_MASK_OFFSET: u32 = 0x28;
const PBB_MASK_OFFSET: u32 = 0x2C;
const PBC_MASK_OFFSET: u32 = 0x30;
const PBD_MASK_OFFSET: u32 = 0x34;

const PM_BASE: usize = 0x400E0000;
const PM_REGS: StaticRef<PmRegisters> = unsafe { StaticRef::new(PM_BASE as *const PmRegisters) };

/// Contains state for the power management peripheral. This includes the
/// configurations for various system clocks and the final frequency that the
/// system is running at.
pub struct PowerManager {
    /// Clock source configuration
    system_clock_source: Cell<SystemClockSource>,

    /// Mask of clocks that are on
    system_on_clocks: Cell<u32>,

    /// Has setup_system_clock been called once
    system_initial_configs: Cell<bool>,
}

pub static mut PM: PowerManager = PowerManager {
    /// Set to the RCSYS by default.
    system_clock_source: Cell::new(SystemClockSource::RcsysAt115kHz),

    system_on_clocks: Cell::new(ClockMask::RCSYS as u32),

    system_initial_configs: Cell::new(false),
};

impl PowerManager {
    /// Sets up the system clock. This should be called as one of the first
    /// lines in the `reset_handler` within the platform's `main.rs`.
    pub unsafe fn setup_system_clock(&self, clock_source: SystemClockSource) {
        if !self.system_initial_configs.get() {
            // For now, always go to PS2 as it enables all core speeds
            // These features are not available in PS1: USB, DFLL, Flash programming/erasing
            bpm::set_power_scaling(bpm::PowerScaling::PS2);

            // Need the 32k RC oscillator for BPM, AST, and DFLL
            bscif::enable_rc32k();

            // Enable HCACHE
            flashcalw::FLASH_CONTROLLER.enable_cache();

            // Enable flash high speed mode, only for PS2
            flashcalw::FLASH_CONTROLLER.enable_high_speed_flash();

            self.system_initial_configs.set(true);
        }

        match clock_source {
            SystemClockSource::RcsysAt115kHz => {
                // No configurations necessary, RCSYS is always on in run mode
                // Set Flash wait state to 0 for <= 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(0);
                // Change the system clock to RCSYS
                select_main_clock(MainClock::RCSYS);
            }

            SystemClockSource::DfllRc32kAt48MHz => {
                // Configure and turn on DFLL at 48MHz
                configure_48mhz_dfll();
                // Set Flash wait state to 1 for > 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(1);
                // Change the system clock to DFLL
                select_main_clock(MainClock::DFLL);
            }

            SystemClockSource::ExternalOscillator {
                frequency,
                startup_mode,
            } => {
                match self.system_clock_source.get() {
                    // If the PLL is running (it uses OSC0 as a reference clock),
                    // temporarily change the system clock to RCSYS to avoid buggy behavior
                    SystemClockSource::PllExternalOscillatorAt48MHz { .. } => {
                        select_main_clock(MainClock::RCSYS);
                    }
                    // Some peripherals (uart,spi) show buggy behavior if the system clock
                    // is directly switched from OSC0 to DFLL - no explanation in documentation
                    SystemClockSource::DfllRc32kAt48MHz => {
                        select_main_clock(MainClock::RCSYS);
                    }
                    _ => {}
                }
                // Configure and turn on OSC0
                configure_external_oscillator(frequency, startup_mode);
                // Set Flash wait state to 0 for <= 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(0);
                // Change the system clock to OSC0
                select_main_clock(MainClock::OSC0);
            }

            SystemClockSource::PllExternalOscillatorAt48MHz {
                frequency,
                startup_mode,
            } => {
                // Configure and turn on PLL at 48MHz
                configure_external_oscillator_pll(frequency, startup_mode);
                // Set Flash wait state to 1 for > 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(1);
                // Change the system clock to PLL
                select_main_clock(MainClock::PLL);
            }

            SystemClockSource::RC80M => {
                // Configure and turn on RC80M
                configure_80mhz_rc();

                // If the 80MHz RC is used as the main clock source, it must be divided by
                //  at least 2 before being used as CPU's clock source
                let cpusel = (*PM_REGS).cpusel.extract();
                unlock(0x00000004);
                (*PM_REGS).cpusel.modify_no_read(
                    cpusel,
                    CpuClockSelect::CPUDIV::SET + CpuClockSelect::CPUSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                // Set Flash wait state to 1 for > 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(1);
                // Change the system clock to RC80M
                select_main_clock(MainClock::RC80M);
            }

            SystemClockSource::RCFAST { frequency } => {
                // Check if RCFAST is already on, in which case temporarily switch the system to RCSYS
                // since RCFAST has to be disabled before its configurations can be changed
                if (PM.system_on_clocks.get() & (ClockMask::RCFAST as u32)) != 0 {
                    select_main_clock(MainClock::RCSYS);
                    scif::disable_rcfast();
                }

                // Some peripherals (uart,spi) show buggy behavior if the system clock
                // is directly switched from RCFAST to DFLL - no explanation in documentation
                match self.system_clock_source.get() {
                    SystemClockSource::DfllRc32kAt48MHz => {
                        select_main_clock(MainClock::RCSYS);
                    }
                    _ => {}
                }

                // Configure and turn on RCFAST at specified frequency
                configure_rcfast(frequency);
                // Set Flash wait state to 0 for <= 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(0);
                // Change the system clock to RCFAST
                select_main_clock(MainClock::RCFAST);
            }

            SystemClockSource::RC1M => {
                match self.system_clock_source.get() {
                    SystemClockSource::DfllRc32kAt48MHz => {
                        select_main_clock(MainClock::RCSYS);
                    }
                    _ => {}
                }

                // Configure and turn on RC1M
                configure_1mhz_rc();
                // Set Flash wait state to 0 for <= 24MHz in PS2
                flashcalw::FLASH_CONTROLLER.set_wait_state(0);
                // Change the system clock to RC1M
                select_main_clock(MainClock::RC1M);
            }
        }

        self.system_clock_source.set(clock_source);
    }

    // Disables the clock passed in as clock_source
    pub unsafe fn disable_system_clock(&self, clock_source: SystemClockSource) {
        // Disable previous clock

        match clock_source {
            SystemClockSource::RcsysAt115kHz => {
                //Rcsys is always on except in sleep modes
            }

            SystemClockSource::ExternalOscillator { .. } => {
                // Only turn off OSC0 if PLL is not using it as a reference clock
                if self.system_on_clocks.get() & (ClockMask::PLL as u32) == 0 {
                    scif::disable_osc_16mhz();
                }
                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::OSC0 as u32));
            }

            SystemClockSource::PllExternalOscillatorAt48MHz { .. } => {
                // Disable PLL
                scif::disable_pll();
                // don't turn off reference clock OSC0 if OSC0 is on
                if self.system_on_clocks.get() & (ClockMask::OSC0 as u32) == 0 {
                    scif::disable_osc_16mhz();
                }
                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::PLL as u32));
            }

            SystemClockSource::DfllRc32kAt48MHz => {
                // Disable DFLL
                scif::disable_dfll_rc32k();
                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::DFLL as u32));
            }

            SystemClockSource::RC80M => {
                // Disable RC80M
                scif::disable_rc_80mhz();

                // Stop dividing the main clock
                let cpusel = (*PM_REGS).cpusel.extract();
                unlock(0x00000004);
                (*PM_REGS).cpusel.modify_no_read(
                    cpusel,
                    CpuClockSelect::CPUDIV::CLEAR + CpuClockSelect::CPUSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                // Stop dividing peripheral clocks
                let pbasel = (*PM_REGS).pbasel.extract();
                unlock(0x0000000C);
                (*PM_REGS).pbasel.modify_no_read(
                    pbasel,
                    PeripheralBusXClockSelect::PBDIV::CLEAR
                        + PeripheralBusXClockSelect::PBSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                let pbbsel = (*PM_REGS).pbbsel.extract();
                unlock(0x00000010);
                (*PM_REGS).pbbsel.modify_no_read(
                    pbbsel,
                    PeripheralBusXClockSelect::PBDIV::CLEAR
                        + PeripheralBusXClockSelect::PBSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                let pbcsel = (*PM_REGS).pbcsel.extract();
                unlock(0x00000014);
                (*PM_REGS).pbcsel.modify_no_read(
                    pbcsel,
                    PeripheralBusXClockSelect::PBDIV::CLEAR
                        + PeripheralBusXClockSelect::PBSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                let pbdsel = (*PM_REGS).pbdsel.extract();
                unlock(0x00000018);
                (*PM_REGS).pbdsel.modify_no_read(
                    pbdsel,
                    PeripheralBusXClockSelect::PBDIV::CLEAR
                        + PeripheralBusXClockSelect::PBSEL::CLEAR,
                );
                while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::RC80M as u32));
            }

            SystemClockSource::RCFAST { .. } => {
                // Disable RCFAST
                scif::disable_rcfast();
                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::RCFAST as u32));
            }

            SystemClockSource::RC1M => {
                // Disable RC1M
                bscif::disable_rc_1mhz();
                let clock_mask = self.system_on_clocks.get();
                self.system_on_clocks
                    .set(clock_mask & !(ClockMask::RC1M as u32));
            }
        }
    }

    // Changes the system clock to the clock passed in as clock_source and disables
    // the previous system clock
    pub unsafe fn change_system_clock(&self, clock_source: SystemClockSource) {
        // If the clock you want to switch to is the current system clock, do nothing
        let prev_clock_source = PM.system_clock_source.get();
        if prev_clock_source == clock_source {
            return;
        }

        // Turn on and switch to the new system clock
        self.setup_system_clock(clock_source);

        // Don't disable RCFAST if the current clock is still RCFAST, just at
        // a different frequency
        match clock_source {
            SystemClockSource::RCFAST { .. } => match prev_clock_source {
                SystemClockSource::RCFAST { .. } => {
                    return;
                }
                _ => {}
            },
            _ => {}
        }
        // Disable the previous system clock
        self.disable_system_clock(prev_clock_source);
    }
}

fn unlock(register_offset: u32) {
    PM_REGS.unlock.set(0xAA000000 | register_offset);
}

fn select_main_clock(clock: MainClock) {
    unlock(0);
    PM_REGS.mcctrl.set(clock as u32);
}

/// Configure the system clock to use the DFLL with the RC32K as the source.
/// Run at 48 MHz.
unsafe fn configure_48mhz_dfll() {
    // Start the DFLL
    scif::setup_dfll_rc32k_48mhz();

    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks.set(clock_mask | ClockMask::DFLL as u32);
}

/// Configure the system clock to use the 16 MHz external crystal directly
unsafe fn configure_external_oscillator(
    frequency: OscillatorFrequency,
    startup_mode: OscillatorStartup,
) {
    // Start the OSC0 if it isn't already in use by the PLL
    if (PM.system_on_clocks.get() & ClockMask::PLL as u32) == 0 {
        match frequency {
            OscillatorFrequency::Frequency16MHz => {
                match startup_mode {
                    OscillatorStartup::FastStart => scif::setup_osc_16mhz_fast_startup(),
                    OscillatorStartup::SlowStart => scif::setup_osc_16mhz_slow_startup(),
                };
            }
        }
    }

    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks.set(clock_mask | ClockMask::OSC0 as u32);
}

/// Configure the system clock to use the PLL with the 16 MHz external crystal
unsafe fn configure_external_oscillator_pll(
    frequency: OscillatorFrequency,
    startup_mode: OscillatorStartup,
) {
    // Start the OSC0 if it isn't already on
    if (PM.system_on_clocks.get() & ClockMask::OSC0 as u32) == 0 {
        match frequency {
            OscillatorFrequency::Frequency16MHz => {
                match startup_mode {
                    OscillatorStartup::FastStart => scif::setup_osc_16mhz_fast_startup(),
                    OscillatorStartup::SlowStart => scif::setup_osc_16mhz_slow_startup(),
                };
            }
        }
    }

    // Start the PLL
    scif::setup_pll_osc_48mhz();

    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks.set(clock_mask | ClockMask::PLL as u32);
}

unsafe fn configure_80mhz_rc() {
    // Start the 80mhz RC oscillator
    scif::setup_rc_80mhz();

    // Divide peripheral clocks so that fCPU >= fAPBx
    let pbasel = (*PM_REGS).pbasel.extract();
    unlock(0x0000000C);
    (*PM_REGS).pbasel.modify_no_read(
        pbasel,
        PeripheralBusXClockSelect::PBDIV::SET + PeripheralBusXClockSelect::PBSEL::CLEAR,
    );
    while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

    let pbbsel = (*PM_REGS).pbbsel.extract();
    unlock(0x00000010);
    (*PM_REGS).pbbsel.modify_no_read(
        pbbsel,
        PeripheralBusXClockSelect::PBDIV::SET + PeripheralBusXClockSelect::PBSEL::CLEAR,
    );
    while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

    let pbcsel = (*PM_REGS).pbcsel.extract();
    unlock(0x00000014);
    (*PM_REGS).pbcsel.modify_no_read(
        pbcsel,
        PeripheralBusXClockSelect::PBDIV::SET + PeripheralBusXClockSelect::PBSEL::CLEAR,
    );
    while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

    let pbdsel = (*PM_REGS).pbdsel.extract();
    unlock(0x00000018);
    (*PM_REGS).pbdsel.modify_no_read(
        pbdsel,
        PeripheralBusXClockSelect::PBDIV::SET + PeripheralBusXClockSelect::PBSEL::CLEAR,
    );
    while (*PM_REGS).sr.matches_all(InterruptOrStatus::CKRDY::CLEAR) {}

    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks
        .set(clock_mask | ClockMask::RC80M as u32);
}

unsafe fn configure_rcfast(frequency: RcfastFrequency) {
    // Start the RCFAST at the specified frequency
    match frequency {
        RcfastFrequency::Frequency4MHz => {
            scif::setup_rcfast_4mhz();
        }
        RcfastFrequency::Frequency8MHz => {
            scif::setup_rcfast_8mhz();
        }
        RcfastFrequency::Frequency12MHz => {
            scif::setup_rcfast_12mhz();
        }
    }

    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks
        .set(clock_mask | ClockMask::RCFAST as u32);
}

unsafe fn configure_1mhz_rc() {
    // Start the RC1M
    bscif::setup_rc_1mhz();
    let clock_mask = PM.system_on_clocks.get();
    PM.system_on_clocks.set(clock_mask | ClockMask::RC1M as u32);
}

pub fn get_system_frequency() -> u32 {
    // Return the current system frequency
    unsafe {
        match PM.system_clock_source.get() {
            SystemClockSource::RcsysAt115kHz => 115200,
            SystemClockSource::DfllRc32kAt48MHz => 48000000,
            SystemClockSource::ExternalOscillator { .. } => 16000000,
            SystemClockSource::PllExternalOscillatorAt48MHz { .. } => 48000000,
            SystemClockSource::RC80M => 40000000,
            SystemClockSource::RCFAST { frequency } => match frequency {
                RcfastFrequency::Frequency4MHz => 4300000,
                RcfastFrequency::Frequency8MHz => 8200000,
                RcfastFrequency::Frequency12MHz => 12000000,
            },
            SystemClockSource::RC1M => 1000000,
        }
    }
}

/// Utility macro to modify clock mask registers
///
/// It takes one of two forms:
///
///     mask_clock!(CLOCK: pm_register | value)
///
/// which performs a logical-or on the existing register value, or
///
///     mask_clock!(CLOCK: pm_register & value)
///
/// which performs a logical-and.
///
/// CLOCK is one of HSB, PBA, PBB, PBC or PBD
///
/// pm_register is one of hsbmask, pbamask, pbbmask, pbcmask or pbdmask.
///
macro_rules! mask_clock {
    ($module:ident : $field:ident | $mask:expr) => {{
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = PM_REGS.$field.get() | ($mask);
        PM_REGS.$field.set(val);
    }};

    ($module:ident : $field:ident & $mask:expr) => {{
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = PM_REGS.$field.get() & ($mask);
        PM_REGS.$field.set(val);
    }};
}

/// Utility macro to get value of clock register. Used to check if a specific
/// clock is enabled or not. See above description of `make_clock!`.
macro_rules! get_clock {
    ($module:ident : $field:ident & $mask:expr) => {{
        unlock(concat_idents!($module, _MASK_OFFSET));
        (PM_REGS.$field.get() & ($mask)) != 0
    }};
}

/// Determines if the chip can safely go into deep sleep without preventing
/// currently active peripherals from operating.
///
/// We look at the PM's clock mask registers and compare them against a set of
/// known masks that include no peripherals that can't operate in deep
/// sleep (or that have no function during sleep). Specifically:
///
///   * HSB may only have clocks for the flash (and PicoCache), APBx bridges, and PDCA on.
///
///   * PBA may only have I2C Slaves on as they can self-wake.
///
///   * PBB may only have clocks for the flash, HRAMC1 (also flash related), and PDCA on.
///
///   * PBC and PBD may have any clocks on.
///
/// This means it is the responsibility of each peripheral to disable it's clock
/// mask whenever it is idle.
///
/// A special note here regarding the PDCA (Peripheral DMA Controller) clock.
/// If the core deep sleeps while a DMA operation is active, it is transparently paused
/// and resumed when the core wakes again. If a peripheral needs a DMA operation to complete
/// before sleeping, the peripheral should inhibit sleep. The rationale here is to allow deep
/// sleep for an I2C Slave peripheral configured to use DMA.
///
/// We also special case GPIO (which is in PBCMASK), and just see if any interrupts are pending
/// through the INTERRUPT_COUNT variable.
pub fn deep_sleep_ready() -> bool {
    // HSB clocks that can be enabled and the core is permitted to enter deep sleep.
    let deep_sleep_hsbmask: FieldValue<u32, ClockMaskHsb::Register> =
        /* added by us */ ClockMaskHsb::PDCA::SET +
        /*     default */ ClockMaskHsb::FLASHCALW::SET +
        /* added by us */ ClockMaskHsb::FLASHCALW_PICOCACHE::SET +
        /*     default */ ClockMaskHsb::APBA_BRIDGE::SET +
        /*     default */ ClockMaskHsb::APBB_BRIDGE::SET +
        /*     default */ ClockMaskHsb::APBC_BRIDGE::SET +
        /*     default */ ClockMaskHsb::APBD_BRIDGE::SET;

    // PBA clocks that can be enabled and the core is permitted to enter deep sleep.
    let deep_sleep_pbamask: FieldValue<u32, ClockMaskPba::Register> =
        /* added by us */ ClockMaskPba::TWIS0::SET +
        /* added by us */ ClockMaskPba::TWIS1::SET;

    // PBB clocks that can be enabled and the core is permitted to enter deep sleep.
    let deep_sleep_pbbmask: FieldValue<u32, ClockMaskPbb::Register> =
        /*     default */ ClockMaskPbb::FLASHCALW::SET +
        /* added by us */ ClockMaskPbb::HRAMC1::SET +
        /* added by us */ ClockMaskPbb::PDCA::SET;

    let hsb = PM_REGS.hsbmask.get() & !deep_sleep_hsbmask.mask() == 0;
    let pba = PM_REGS.pbamask.get() & !deep_sleep_pbamask.mask() == 0;
    let pbb = PM_REGS.pbbmask.get() & !deep_sleep_pbbmask.mask() == 0;
    let gpio = gpio::INTERRUPT_COUNT.load(Ordering::Relaxed) == 0;
    hsb && pba && pbb && gpio
}

impl ClockInterface for Clock {
    fn is_enabled(&self) -> bool {
        match self {
            &Clock::HSB(v) => get_clock!(HSB: hsbmask & (1 << (v as u32))),
            &Clock::PBA(v) => get_clock!(PBA: pbamask & (1 << (v as u32))),
            &Clock::PBB(v) => get_clock!(PBB: pbbmask & (1 << (v as u32))),
            &Clock::PBC(v) => get_clock!(PBC: pbcmask & (1 << (v as u32))),
            &Clock::PBD(v) => get_clock!(PBD: pbdmask & (1 << (v as u32))),
        }
    }

    fn enable(&self) {
        match self {
            &Clock::HSB(v) => mask_clock!(HSB: hsbmask | 1 << (v as u32)),
            &Clock::PBA(v) => mask_clock!(PBA: pbamask | 1 << (v as u32)),
            &Clock::PBB(v) => mask_clock!(PBB: pbbmask | 1 << (v as u32)),
            &Clock::PBC(v) => mask_clock!(PBC: pbcmask | 1 << (v as u32)),
            &Clock::PBD(v) => mask_clock!(PBD: pbdmask | 1 << (v as u32)),
        }
    }

    fn disable(&self) {
        match self {
            &Clock::HSB(v) => mask_clock!(HSB: hsbmask & !(1 << (v as u32))),
            &Clock::PBA(v) => mask_clock!(PBA: pbamask & !(1 << (v as u32))),
            &Clock::PBB(v) => mask_clock!(PBB: pbbmask & !(1 << (v as u32))),
            &Clock::PBC(v) => mask_clock!(PBC: pbcmask & !(1 << (v as u32))),
            &Clock::PBD(v) => mask_clock!(PBD: pbdmask & !(1 << (v as u32))),
        }
    }
}

pub fn enable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | 1 << (v as u32)),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | 1 << (v as u32)),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | 1 << (v as u32)),
        Clock::PBC(v) => mask_clock!(PBC: pbcmask | 1 << (v as u32)),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | 1 << (v as u32)),
    }
}

pub fn disable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask & !(1 << (v as u32))),
        Clock::PBA(v) => mask_clock!(PBA: pbamask & !(1 << (v as u32))),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask & !(1 << (v as u32))),
        Clock::PBC(v) => mask_clock!(PBC: pbcmask & !(1 << (v as u32))),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask & !(1 << (v as u32))),
    }
}

pub fn is_clock_enabled(clock: Clock) -> bool {
    match clock {
        Clock::HSB(v) => get_clock!(HSB: hsbmask & (1 << (v as u32))),
        Clock::PBA(v) => get_clock!(PBA: pbamask & (1 << (v as u32))),
        Clock::PBB(v) => get_clock!(PBB: pbbmask & (1 << (v as u32))),
        Clock::PBC(v) => get_clock!(PBC: pbcmask & (1 << (v as u32))),
        Clock::PBD(v) => get_clock!(PBD: pbdmask & (1 << (v as u32))),
    }
}
