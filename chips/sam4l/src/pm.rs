use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
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
    ier: VolatileCell<u32>, // 0xC0
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    _reserved6: [VolatileCell<u32>; 34], // 0x100
    ppcr: VolatileCell<u32>, // 0x160
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

#[repr(C, packed)]
struct BscifRegisters {
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    pclksr: VolatileCell<u32>,
    unlock: VolatileCell<u32>,
    cscr: VolatileCell<u32>,
    oscctrl32: VolatileCell<u32>,
    rc32kcr: VolatileCell<u32>,
    rc32ktune: VolatileCell<u32>,
    bod33ctrl: VolatileCell<u32>,
    bod33level: VolatileCell<u32>,
    bod33sampling: VolatileCell<u32>,
    bod18ctrl: VolatileCell<u32>,
    bot18level: VolatileCell<u32>,
    bod18sampling: VolatileCell<u32>,
    vregcr: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 4],
    rc1mcr: VolatileCell<u32>,
    _reserved2: VolatileCell<u32>,
    bgctrl: VolatileCell<u32>,
    bgsr: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 4],
    br0: VolatileCell<u32>,
    br1: VolatileCell<u32>,
    br2: VolatileCell<u32>,
    br3: VolatileCell<u32>,
    _reserved4: [VolatileCell<u32>; 215],
    brifbversion: VolatileCell<u32>,
    bgrefifbversion: VolatileCell<u32>,
    vregifgversion: VolatileCell<u32>,
    bodifcversion: VolatileCell<u32>,
    rc32kifbversion: VolatileCell<u32>,
    osc32ifaversion: VolatileCell<u32>,
    version: VolatileCell<u32>,
}

#[repr(C, packed)]
struct ScifRegisters {
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    isr: VolatileCell<u32>,
    icr: VolatileCell<u32>,
    pclksr: VolatileCell<u32>,
    unlock: VolatileCell<u32>,
    cscr: VolatileCell<u32>,
    oscctrl0: VolatileCell<u32>,
    pll0: VolatileCell<u32>,
    dfll0conf: VolatileCell<u32>,
    dfll0val: VolatileCell<u32>,
    dfll0mul: VolatileCell<u32>,
    dfll0step: VolatileCell<u32>,
    dfll0ssg: VolatileCell<u32>,
    dfll0ratio: VolatileCell<u32>,
    dfll0sync: VolatileCell<u32>,
    rccr: VolatileCell<u32>,
    rcfastcfg: VolatileCell<u32>,
    rcfastsr: VolatileCell<u32>,
    rc80mcr: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 4],
    hrpcr: VolatileCell<u32>,
    fpcr: VolatileCell<u32>,
    fpmul: VolatileCell<u32>,
    fpdiv: VolatileCell<u32>,
    gcctrl0: VolatileCell<u32>,
    gcctrl1: VolatileCell<u32>,
    gcctrl2: VolatileCell<u32>,
    gcctrl3: VolatileCell<u32>,
    gcctrl4: VolatileCell<u32>,
    gcctrl5: VolatileCell<u32>,
    gcctrl6: VolatileCell<u32>,
    gcctrl7: VolatileCell<u32>,
    gcctrl8: VolatileCell<u32>,
    gcctrl9: VolatileCell<u32>,
    gcctrl10: VolatileCell<u32>,
    gcctrl11: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 205],
    rcfastversion: VolatileCell<u32>,
    gclkprescversion: VolatileCell<u32>,
    pllifaversion: VolatileCell<u32>,
    oscifaversion: VolatileCell<u32>,
    dfllifbversion: VolatileCell<u32>,
    rcoscifaversion: VolatileCell<u32>,
    _reserved3: VolatileCell<u32>,
    rc80mversion: VolatileCell<u32>,
    gclkversion: VolatileCell<u32>,
    version: VolatileCell<u32>,
}

#[repr(C, packed)]
struct FlashcalwRegisters {
    fcr: VolatileCell<u32>,
    fcmd: VolatileCell<u32>,
    fsr: VolatileCell<u32>,
    fpr: VolatileCell<u32>,
    fvr: VolatileCell<u32>,
    fgpfrhi: VolatileCell<u32>,
    fgpfrlo: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 251],
    ctrl: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 4],
    maint0: VolatileCell<u32>,
    maint1: VolatileCell<u32>,
    mcfg: VolatileCell<u32>,
    men: VolatileCell<u32>,
    mctrl: VolatileCell<u32>,
    msr: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 49],
    pvr: VolatileCell<u32>,
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

#[derive(Copy,Clone)]
pub enum Clock {
    HSB(HSBClock),
    PBA(PBAClock),
    PBB(PBBClock),
    PBD(PBDClock),
}

#[derive(Copy,Clone)]
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

#[derive(Copy,Clone)]
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

#[derive(Copy,Clone)]
pub enum PBBClock {
    FLASHCALW,
    HRAMC1,
    HMATRIX,
    PDCA,
    CRCCU,
    USBC,
    PEVC,
}

#[derive(Copy,Clone)]
pub enum PBDClock {
    BPM,
    BSCIF,
    AST,
    WDT,
    EIC,
    PICOUART,
}

/// Which source the system clock should be generated from.
pub enum SystemClockSource {
    /// Use the internal digital frequency locked loop (DFLL) sourced from
    /// the internal RC32K clock. Note this typically requires calibration
    /// of the RC32K to have a consistent clock.
    DfllRc32k,

    /// Use an external crystal oscillator as the direct source for the
    /// system clock.
    ExternalOscillator,

    /// Use an external crystal oscillator as the input to the internal
    /// PLL for the system clock. This expects a 16 MHz crystal.
    ExternalOscillatorPll,
}

const PM_BASE: usize = 0x400E0000;
const BSCIF_BASE: usize = 0x400F0400;
const SCIF_BASE: usize = 0x400E0800;
const FLASHCALW_BASE: usize = 0x400A0000;

const HSB_MASK_OFFSET: u32 = 0x24;
const PBA_MASK_OFFSET: u32 = 0x28;
const PBB_MASK_OFFSET: u32 = 0x2C;
const PBD_MASK_OFFSET: u32 = 0x34;

static mut PM: *mut PmRegisters = PM_BASE as *mut PmRegisters;
static mut BSCIF: *mut BscifRegisters = BSCIF_BASE as *mut BscifRegisters;
static mut SCIF: *mut ScifRegisters = SCIF_BASE as *mut ScifRegisters;
static mut FLASHCALW: *mut FlashcalwRegisters = FLASHCALW_BASE as *mut FlashcalwRegisters;

static mut SYSTEM_FREQUENCY: VolatileCell<u32> = VolatileCell::new(0);

unsafe fn unlock(register_offset: u32) {
    (*PM).unlock.set(0xAA000000 | register_offset);
}

unsafe fn select_main_clock(clock: MainClock) {
    unlock(0);
    (*PM).mcctrl.set(clock as u32);
}

/// Enable HCACHE
unsafe fn enable_cache() {
    enable_clock(Clock::HSB(HSBClock::FLASHCALWP));
    enable_clock(Clock::PBB(PBBClock::HRAMC1));
    // Enable cache
    (*FLASHCALW).ctrl.set(0x01);

    // Wait for the cache controller to be enabled.
    while (*FLASHCALW).sr.get() & (1 << 0) == 0 {}
}

/// Configure high-speed flash mode. This is taken from the ASF code
unsafe fn enable_high_speed_flash() {
    // Since we are running at a fast speed we have to set a clock delay
    // for flash, as well as enable fast flash mode.
    let flashcalw_fcr = (*FLASHCALW).fcr.get();
    (*FLASHCALW).fcr.set(flashcalw_fcr | (1 << 6));

    // Enable high speed mode for flash
    let flashcalw_fcmd = (*FLASHCALW).fcmd.get();
    let flashcalw_fcmd_new1 = flashcalw_fcmd & (!(0x3F << 0));
    let flashcalw_fcmd_new2 = flashcalw_fcmd_new1 | (0xA5 << 24) | (0x10 << 0);
    (*FLASHCALW).fcmd.set(flashcalw_fcmd_new2);

    // And wait for the flash to be ready
    while (*FLASHCALW).fsr.get() & (1 << 0) == 0 {}
}

/// Setup the internal 32kHz RC oscillator.
unsafe fn enable_rc32k() {
    let bscif_rc32kcr = (*BSCIF).rc32kcr.get();
    // Unlock the BSCIF::RC32KCR register
    (*BSCIF).unlock.set(0xAA000024);
    // Write the BSCIF::RC32KCR register.
    // Enable the generic clock source, the temperature compensation, and the
    // 32k output.
    (*BSCIF).rc32kcr.set(bscif_rc32kcr | (1 << 2) | (1 << 1) | (1 << 0));
    // Wait for it to be ready, although it feels like this won't do anything
    while (*BSCIF).rc32kcr.get() & (1 << 0) == 0 {}

    // Load magic calibration value for the 32KHz RC oscillator
    //
    // Unlock the BSCIF::RC32KTUNE register
    (*BSCIF).unlock.set(0xAA000028);
    // Write the BSCIF::RC32KTUNE register
    (*BSCIF).rc32ktune.set(0x001d0015);
}

/// Configure the system clock to use the DFLL with the RC32K as the source.
/// Run at 48 MHz.
unsafe fn configure_48mhz_dfll() {
    // Enable HCACHE
    enable_cache();

    // Check to see if the DFLL is already setup.
    //
    if (((*SCIF).dfll0conf.get() & 0x03) == 0) || (((*SCIF).pclksr.get() & (1 << 2)) == 0) {

        // Enable the GENCLK_SRC_RC32K
        enable_rc32k();

        // Next init closed loop mode.
        //
        // Must do a SCIF sync before reading the SCIF register
        (*SCIF).dfll0sync.set(0x01);
        // Wait for it to be ready
        while (*SCIF).pclksr.get() & (1 << 3) == 0 {}

        // Read the current DFLL settings
        let scif_dfll0conf = (*SCIF).dfll0conf.get();
        // Set the new values
        //                                        enable     closed loop
        let scif_dfll0conf_new1 = scif_dfll0conf | (1 << 0) | (1 << 1);
        let scif_dfll0conf_new2 = scif_dfll0conf_new1 & (!(3 << 16));
        // frequency range 2
        let scif_dfll0conf_new3 = scif_dfll0conf_new2 | (2 << 16);
        // Enable the general clock. Yeah getting this fields is complicated.
        //                 enable     RC32K       no divider
        let scif_gcctrl0 = (1 << 0) | (13 << 8) | (0 << 1) | (0 << 16);
        (*SCIF).gcctrl0.set(scif_gcctrl0);

        // Setup DFLL. Must wait after every operation for the ready bit to go high.
        // First, enable dfll apparently
        // unlock dfll0conf
        (*SCIF).unlock.set(0xAA000028);
        // enable
        (*SCIF).dfll0conf.set(0x01);
        while (*SCIF).pclksr.get() & (1 << 3) == 0 {}
        // Set step values
        // unlock
        (*SCIF).unlock.set(0xAA000034);
        // 4, 4
        (*SCIF).dfll0step.set((4 << 0) | (4 << 16));
        while (*SCIF).pclksr.get() & (1 << 3) == 0 {}
        // Set multiply value
        // unlock
        (*SCIF).unlock.set(0xAA000030);
        // 1464 = 48000000 / 32768
        (*SCIF).dfll0mul.set(1464);
        while (*SCIF).pclksr.get() & (1 << 3) == 0 {}
        // Set SSG value
        // unlock
        (*SCIF).unlock.set(0xAA000038);
        // just set to zero to disable
        (*SCIF).dfll0ssg.set(0);
        while (*SCIF).pclksr.get() & (1 << 3) == 0 {}
        // Set actual configuration
        // unlock
        (*SCIF).unlock.set(0xAA000028);
        // we already prepared this value
        (*SCIF).dfll0conf.set(scif_dfll0conf_new3);

        // Now wait for it to be ready (DFLL0LOCKF)
        while (*SCIF).pclksr.get() & (1 << 2) == 0 {}
    }

    // Since we are running at a fast speed we have to set a clock delay
    // for flash, as well as enable fast flash mode.
    enable_high_speed_flash();

    // Choose the main clock
    select_main_clock(MainClock::DFLL);
}

/// Configure the system clock to use the DFLL with the 16 MHz external crystal.
unsafe fn configure_external_oscillator() {
    // Use the cache
    enable_cache();

    // Need the 32k RC oscillator for things like BPM module and AST.
    enable_rc32k();

    // Enable the OSC0
    (*SCIF).unlock.set(0xAA000020);
    // enable, 557 us startup time, gain level 4 (sortof), is crystal.
    (*SCIF).oscctrl0.set((1 << 16) | (1 << 8) | (4 << 1) | (1 << 0));
    // Wait for oscillator to be ready
    while (*SCIF).pclksr.get() & (1 << 0) == 0 {}

    // Go to high speed flash mode
    enable_high_speed_flash();

    // Set the main clock to be the external oscillator
    select_main_clock(MainClock::OSC0);
}

/// Configure the system clock to use the DFLL with the 16 MHz external crystal.
unsafe fn configure_external_oscillator_pll() {
    // Use the cache
    enable_cache();

    // Need the 32k RC oscillator for things like BPM module and AST.
    enable_rc32k();

    // Enable the OSC0
    (*SCIF).unlock.set(0xAA000020);
    // enable, 557 us startup time, gain level 4 (sortof), is crystal.
    (*SCIF).oscctrl0.set((1 << 16) | (1 << 8) | (4 << 1) | (1 << 0));
    // Wait for oscillator to be ready
    while (*SCIF).pclksr.get() & (1 << 0) == 0 {}

    // Setup the PLL
    // Enable the PLL0 register
    (*SCIF).unlock.set(0xAA000024);
    // Maximum startup time, multiply by 5, divide=1, divide output by 2, enable.
    (*SCIF).pll0.set((0x3F << 24) | (5 << 16) | (1 << 8) | (1 << 4) | (1 << 0));
    // Wait for the PLL to be ready
    while (*SCIF).pclksr.get() & (1 << 6) == 0 {}

    // Go to high speed flash mode
    enable_high_speed_flash();

    // Set the main clock to be the PLL
    select_main_clock(MainClock::PLL);
}

pub unsafe fn setup_system_clock(clock_source: SystemClockSource, frequency: u32) {
    SYSTEM_FREQUENCY.set(frequency);

    match clock_source {
        SystemClockSource::DfllRc32k => {
            configure_48mhz_dfll();
        }

        SystemClockSource::ExternalOscillator => {
            configure_external_oscillator();
        }

        SystemClockSource::ExternalOscillatorPll => {
            configure_external_oscillator_pll();
        }
    }
}

pub unsafe fn get_system_frequency() -> u32 {
    SYSTEM_FREQUENCY.get()
}

macro_rules! mask_clock {
    ($module:ident: $field:ident | $mask:expr) => ({
        unlock(concat_idents!($module, _MASK_OFFSET));
        let val = (*PM).$field.get() | ($mask);
        (*PM).$field.set(val);
    });
}

pub unsafe fn enable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | 1 << (v as u32)),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | 1 << (v as u32)),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | 1 << (v as u32)),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | 1 << (v as u32)),
    }
}

pub unsafe fn disable_clock(clock: Clock) {
    match clock {
        Clock::HSB(v) => mask_clock!(HSB: hsbmask | !(1 << (v as u32))),
        Clock::PBA(v) => mask_clock!(PBA: pbamask | !(1 << (v as u32))),
        Clock::PBB(v) => mask_clock!(PBB: pbbmask | !(1 << (v as u32))),
        Clock::PBD(v) => mask_clock!(PBD: pbdmask | !(1 << (v as u32))),
    }
}
