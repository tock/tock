//! Implementation of the power manager (PM) peripheral.

use bpm;
use bscif;
use core::cell::Cell;
use core::sync::atomic::Ordering;
use flashcalw;
use gpio;
use kernel::common::VolatileCell;
use scif;

#[repr(C)]
struct PmRegisters {
    mcctrl: VolatileCell<u32>,
    cpusel: VolatileCell<u32>,
    _reserved1: VolatileCell<u32>,
    pbasel: VolatileCell<u32>,
    pbbsel: VolatileCell<u32>,
    pbcsel: VolatileCell<u32>,
    pbdsel: VolatileCell<u32>,
    _reserved2: VolatileCell<u32>,
    cpumask: VolatileCell<u32>, // 0x020
    hsbmask: VolatileCell<u32>,
    pbamask: VolatileCell<u32>,
    pbbmask: VolatileCell<u32>,
    pbcmask: VolatileCell<u32>,
    pbdmask: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 2],
    pbadivmask: VolatileCell<u32>, // 0x040
    _reserved4: [VolatileCell<u32>; 4],
    cfdctrl: VolatileCell<u32>,
    unlock: VolatileCell<u32>,
    _reserved5: [VolatileCell<u32>; 25], // 0x60
    ier: VolatileCell<u32>,              // 0xC0
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    _reserved6: [VolatileCell<u32>; 34], // 0x100
    ppcr: VolatileCell<u32>,             // 0x160
    _reserved7: [VolatileCell<u32>; 7],
    rcause: VolatileCell<u32>, // 0x180
    wcause: VolatileCell<u32>,
    awen: VolatileCell<u32>,
    protctrl: VolatileCell<u32>,
    _reserved8: VolatileCell<u32>,
    fastsleep: VolatileCell<u32>,
    _reserved9: [VolatileCell<u32>; 152],
    config: VolatileCell<u32>, // 0x200
    version: VolatileCell<u32>,
}

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
#[derive(Copy, Clone, Debug)]
pub enum OscillatorFrequency {
    /// 16 MHz external oscillator
    Frequency16MHz,
}

/// Configuration for the startup time of the external oscillator. In practice
/// we have found that some boards work with a short startup time, while others
/// need a slow start in order to properly wake from sleep. In general, we find
/// that for systems that do not work, at fast speed, they will hang or panic
/// after several entries into WAIT mode.
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub enum SystemClockSource {
    /// Use the RCSYS clock (which the system starts up on anyways). Final
    /// system frequency will be 115 kHz. Note that while this is the default,
    /// Tock is NOT guaranteed to work on this setting and will likely fail.
    RcsysAt115kHz,

    /// Use the internal digital frequency locked loop (DFLL) sourced from
    /// the internal RC32K clock. Note this typically requires calibration
    /// of the RC32K to have a consistent clock. Final frequency of 48 MHz.
    DfllRc32kAt48MHz,

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
}

const PM_BASE: usize = 0x400E0000;

const HSB_MASK_OFFSET: u32 = 0x24;
const PBA_MASK_OFFSET: u32 = 0x28;
const PBB_MASK_OFFSET: u32 = 0x2C;
const PBC_MASK_OFFSET: u32 = 0x30;
const PBD_MASK_OFFSET: u32 = 0x34;

static mut PM_REGS: *mut PmRegisters = PM_BASE as *mut PmRegisters;

/// Contains state for the power management peripheral. This includes the
/// configurations for various system clocks and the final frequency that the
/// system is running at.
pub struct PowerManager {
    /// Frequency at which the system clock is running.
    system_frequency: Cell<u32>,

    /// Clock source configuration
    system_clock_source: Cell<SystemClockSource>,
}

pub static mut PM: PowerManager = PowerManager {
    /// Set to the RCSYS frequency by default (115 kHz).
    system_frequency: Cell::new(115000),

    /// Set to the RCSYS by default.
    system_clock_source: Cell::new(SystemClockSource::RcsysAt115kHz),
};

impl PowerManager {
    /// Sets up the system clock. This should be called as one of the first
    /// lines in the `reset_handler` within the platform's `main.rs`.
    pub unsafe fn setup_system_clock(&self, clock_source: SystemClockSource) {
        // save configuration
        self.system_clock_source.set(clock_source);

        // For now, always go to PS2 as it enables all core speeds
        bpm::set_power_scaling(bpm::PowerScaling::PS2);

        match clock_source {
            SystemClockSource::RcsysAt115kHz => {
                // no configurations necessary, already running off the RCSYS
                self.system_frequency.set(115000);
            }

            SystemClockSource::DfllRc32kAt48MHz => {
                configure_48mhz_dfll();
                self.system_frequency.set(48000000);
            }

            SystemClockSource::ExternalOscillator {
                frequency,
                startup_mode,
            } => {
                configure_external_oscillator(frequency, startup_mode);
                match frequency {
                    OscillatorFrequency::Frequency16MHz => self.system_frequency.set(16000000),
                };
            }

            SystemClockSource::PllExternalOscillatorAt48MHz {
                frequency,
                startup_mode,
            } => {
                configure_external_oscillator_pll(frequency, startup_mode);
                self.system_frequency.set(48000000);
            }
        }
    }
}

unsafe fn unlock(register_offset: u32) {
    (*PM_REGS).unlock.set(0xAA000000 | register_offset);
}

unsafe fn select_main_clock(clock: MainClock) {
    unlock(0);
    (*PM_REGS).mcctrl.set(clock as u32);
}

/// Configure the system clock to use the DFLL with the RC32K as the source.
/// Run at 48 MHz.
unsafe fn configure_48mhz_dfll() {
    // Enable HCACHE
    flashcalw::FLASH_CONTROLLER.enable_cache();

    // start the dfll
    scif::setup_dfll_rc32k_48mhz();

    // Since we are running at a fast speed we have to set a clock delay
    // for flash, as well as enable fast flash mode.
    flashcalw::FLASH_CONTROLLER.enable_high_speed_flash();

    // Choose the main clock
    select_main_clock(MainClock::DFLL);
}

/// Configure the system clock to use the 16 MHz external crystal directly
unsafe fn configure_external_oscillator(
    frequency: OscillatorFrequency,
    startup_mode: OscillatorStartup,
) {
    // Use the cache
    flashcalw::FLASH_CONTROLLER.enable_cache();

    // Need the 32k RC oscillator for things like BPM module and AST.
    bscif::enable_rc32k();

    // start the external oscillator
    match frequency {
        OscillatorFrequency::Frequency16MHz => {
            match startup_mode {
                OscillatorStartup::FastStart => scif::setup_osc_16mhz_fast_startup(),
                OscillatorStartup::SlowStart => scif::setup_osc_16mhz_slow_startup(),
            };
        }
    }

    // Go to high speed flash mode
    flashcalw::FLASH_CONTROLLER.enable_high_speed_flash();

    // Set the main clock to be the external oscillator
    select_main_clock(MainClock::OSC0);
}

/// Configure the system clock to use the PLL with the 16 MHz external crystal
unsafe fn configure_external_oscillator_pll(
    frequency: OscillatorFrequency,
    startup_mode: OscillatorStartup,
) {
    // Use the cache
    flashcalw::FLASH_CONTROLLER.enable_cache();

    // Need the 32k RC oscillator for things like BPM module and AST.
    bscif::enable_rc32k();

    // start the external oscillator
    match frequency {
        OscillatorFrequency::Frequency16MHz => {
            match startup_mode {
                OscillatorStartup::FastStart => scif::setup_osc_16mhz_fast_startup(),
                OscillatorStartup::SlowStart => scif::setup_osc_16mhz_slow_startup(),
            };
        }
    }

    // Setup the PLL
    scif::setup_pll_osc_48mhz();

    // Go to high speed flash mode
    flashcalw::FLASH_CONTROLLER.enable_high_speed_flash();

    // Set the main clock to be the PLL
    select_main_clock(MainClock::PLL);
}

pub fn get_system_frequency() -> u32 {
    unsafe { PM.system_frequency.get() }
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
    ($module:ident: $field:ident | $mask:expr) => ({
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = (*PM_REGS).$field.get() | ($mask);
        (*PM_REGS).$field.set(val);
    });

    ($module:ident: $field:ident & $mask:expr) => ({
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = (*PM_REGS).$field.get() & ($mask);
        (*PM_REGS).$field.set(val);
    });
}

/// Utility macro to get value of clock register. Used to check if a specific
/// clock is enabled or not. See above description of `make_clock!`.
macro_rules! get_clock {
    ($module:ident: $field:ident & $mask:expr) => ({
        unlock(concat_idents!($module, _MASK_OFFSET));
        ((*PM_REGS).$field.get() & ($mask)) != 0
    });
}

// Clock masks that allow us to go into deep sleep without disabling any active
// peripherals.

// FLASHCALW clocks and APBx clocks are allowed
//
// This is identical to the reset value of the HSBMASK except it allows the
// PicoCache RAM clock to be on as well.
const DEEP_SLEEP_HSBMASK: u32 = 0x1e6;

// No clocks allowed on PBA
const DEEP_SLEEP_PBAMASK: u32 = 0x0;

// FLASHCALW and HRAMC1 clocks allowed
//
// This is identical to the reset value of the PBBMASK except it allows the
// flash's HRAMC1 clock as well.
const DEEP_SLEEP_PBBMASK: u32 = 0x3;

/// Determines if the chip can safely go into deep sleep without preventing
/// currently active peripherals from operating.
///
/// We look at the PM's clock mask registers and compare them against a set of
/// known masks that include no peripherals that can't operate in deep
/// sleep (or that have no function during sleep). Specifically:
///
///   * HSB may only have clocks for the flash (and PicoCache) and APBx bridges on.
///
///   * PBA may not have _any_ clocks on.
///
///   * PBB may only have clocks for the flash and HRAMC1 (also flash related) on.
///
///   * PBC and PBD may have any clocks on.
///
/// This means it is the responsibility of each peripheral to disable it's clock
/// mask whenever it is idle.
///
/// We also special case GPIO (which is in PBCMASK), and just see if any interrupts are pending
/// through the INTERRUPT_COUNT variable.
pub fn deep_sleep_ready() -> bool {
    unsafe {
        (*PM_REGS).hsbmask.get() & !(DEEP_SLEEP_HSBMASK) == 0
            && (*PM_REGS).pbamask.get() & !(DEEP_SLEEP_PBAMASK) == 0
            && (*PM_REGS).pbbmask.get() & !(DEEP_SLEEP_PBBMASK) == 0
            && gpio::INTERRUPT_COUNT.load(Ordering::Relaxed) == 0
    }
}

pub unsafe fn enable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | 1 << (v as u32)),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | 1 << (v as u32)),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | 1 << (v as u32)),
        Clock::PBC(v) => mask_clock!(PBC: pbcmask | 1 << (v as u32)),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | 1 << (v as u32)),
    }
}

pub unsafe fn disable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask & !(1 << (v as u32))),
        Clock::PBA(v) => mask_clock!(PBA: pbamask & !(1 << (v as u32))),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask & !(1 << (v as u32))),
        Clock::PBC(v) => mask_clock!(PBC: pbcmask & !(1 << (v as u32))),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask & !(1 << (v as u32))),
    }
}

pub unsafe fn is_clock_enabled(clock: Clock) -> bool {
    match clock {
        Clock::HSB(v) => get_clock!(HSB: hsbmask & (1 << (v as u32))),
        Clock::PBA(v) => get_clock!(PBA: pbamask & (1 << (v as u32))),
        Clock::PBB(v) => get_clock!(PBB: pbbmask & (1 << (v as u32))),
        Clock::PBC(v) => get_clock!(PBC: pbcmask & (1 << (v as u32))),
        Clock::PBD(v) => get_clock!(PBD: pbdmask & (1 << (v as u32))),
    }
}
