//! Implementation of the System Control peripheral.

use core::cell::Cell;
use kernel::common::cells::VolatileCell;

#[derive(Copy, Clone, Debug)]
pub enum Clock {
    TIMER(RCGCTIMER),
    GPIO(RCGCGPIO),
    UART(RCGCUART),
}

#[derive(Copy, Clone, Debug)]
pub enum RCGCTIMER {
    TIMER0,
    TIMER1,
    TIMER2,
    TIMER3,
    TIMER4,
    TIMER5,
    TIMER6,
    TIMER7,
}

#[derive(Copy, Clone, Debug)]
pub enum RCGCGPIO {
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOD,
    GPIOE,
    GPIOF,
    GPIOG,
    GPIOH,
    GPIOJ,
    GPIOK,
    GPIOL,
    GPIOM,
    GPION,
    GPIOP,
    GPIOQ,
}

#[derive(Copy, Clone, Debug)]
pub enum RCGCUART {
    UART0,
    UART1,
    UART2,
    UART3,
    UART4,
    UART5,
    UART6,
    UART7,
}

#[repr(C)]
struct Registers {
    did0: VolatileCell<u32>,
    did1: VolatileCell<u32>,
    _reserved0: [u32; 12],
    ptboctl: VolatileCell<u32>,
    _reserved1: [u32; 5],
    ris: VolatileCell<u32>,
    imc: VolatileCell<u32>,
    misc: VolatileCell<u32>,
    resc: VolatileCell<u32>,
    pwrtc: VolatileCell<u32>,
    nmic: VolatileCell<u32>,
    _reserved2: [u32; 5],
    moscctl: VolatileCell<u32>,
    _reserved3: [u32; 12],
    rsclkcfg: VolatileCell<u32>,
    _reserved4: [u32; 3],
    memtim0: VolatileCell<u32>,
    _reserved5: [u32; 29],
    altclkcfg: VolatileCell<u32>,
    _reserved6: [u32; 2],
    dsclkcfg: VolatileCell<u32>,
    divsclk: VolatileCell<u32>,
    sysprop: VolatileCell<u32>,
    piosccal: VolatileCell<u32>,
    pioscstat: VolatileCell<u32>,
    _reserved7: [u32; 2],
    pllfreq0: VolatileCell<u32>,
    pllfreq1: VolatileCell<u32>,
    pllstat: VolatileCell<u32>,
    _reserved8: [u32; 7],
    slppwrcfg: VolatileCell<u32>,
    dslppwrcfg: VolatileCell<u32>,
    _reserved9: [u32; 4],
    nvmstat: VolatileCell<u32>,
    _reserved10: [u32; 4],
    ldospctl: VolatileCell<u32>,
    _reserved11: [u32; 1],
    ldodpctl: VolatileCell<u32>,
    _reserved12: [u32; 6],
    resbehavctl: VolatileCell<u32>,
    _reserved13: [u32; 6],
    hssr: VolatileCell<u32>,
    _reserved14: [u32; 34],
    usbpds: VolatileCell<u32>,
    usbmpc: VolatileCell<u32>,
    emacpds: VolatileCell<u32>,
    emacmpc: VolatileCell<u32>,
    _reserved15: [u32; 28],
    ppwd: VolatileCell<u32>,
    pptimer: VolatileCell<u32>,
    ppgpio: VolatileCell<u32>,
    ppdma: VolatileCell<u32>,
    ppepi: VolatileCell<u32>,
    pphib: VolatileCell<u32>,
    ppuart: VolatileCell<u32>,
    ppssi: VolatileCell<u32>,
    ppi2c: VolatileCell<u32>,
    _reserved16: [u32; 1],
    ppusb: VolatileCell<u32>,
    _reserved17: [u32; 1],
    ppephy: VolatileCell<u32>,
    ppcan: VolatileCell<u32>,
    ppadc: VolatileCell<u32>,
    ppacmp: VolatileCell<u32>,
    pppwm: VolatileCell<u32>,
    ppqei: VolatileCell<u32>,
    pplpc: VolatileCell<u32>,
    _reserved18: [u32; 1],
    pppeci: VolatileCell<u32>,
    ppfan: VolatileCell<u32>,
    ppeeprom: VolatileCell<u32>,
    ppwtimer: VolatileCell<u32>,
    _reserved19: [u32; 4],
    pprts: VolatileCell<u32>,
    ppccm: VolatileCell<u32>,
    _reserved20: [u32; 6],
    pplcd: VolatileCell<u32>,
    _reserved21: [u32; 1],
    ppowire: VolatileCell<u32>,
    ppemac: VolatileCell<u32>,
    _reserved22: [u32; 1],
    pphim: VolatileCell<u32>,
    _reserved23: [u32; 86],
    srwd: VolatileCell<u32>,
    srtimer: VolatileCell<u32>,
    srgpio: VolatileCell<u32>,
    srdma: VolatileCell<u32>,
    srepi: VolatileCell<u32>,
    srhib: VolatileCell<u32>,
    sruart: VolatileCell<u32>,
    srssi: VolatileCell<u32>,
    sri2c: VolatileCell<u32>,
    _reserved24: [u32; 1],
    srusb: VolatileCell<u32>,
    _reserved25: [u32; 1],
    srephy: VolatileCell<u32>,
    srcan: VolatileCell<u32>,
    sradc: VolatileCell<u32>,
    sracmp: VolatileCell<u32>,
    srpwm: VolatileCell<u32>,
    srqei: VolatileCell<u32>,
    _reserved26: [u32; 4],
    sreeprom: VolatileCell<u32>,
    _reserved27: [u32; 6],
    srccm: VolatileCell<u32>,
    _reserved28: [u32; 9],
    sremac: VolatileCell<u32>,
    _reserved29: [u32; 24],
    rcgcwd: VolatileCell<u32>,
    rcgctimer: VolatileCell<u32>,
    rcgcgpio: VolatileCell<u32>,
    rcgcdma: VolatileCell<u32>,
    rcgcepi: VolatileCell<u32>,
    rcgchib: VolatileCell<u32>,
    rcgcuart: VolatileCell<u32>,
    rcgcssi: VolatileCell<u32>,
    rcgci2c: VolatileCell<u32>,
    _reserved30: [u32; 1],
    rcgcusb: VolatileCell<u32>,
    _reserved31: [u32; 1],
    rcgcephy: VolatileCell<u32>,
    rcgccan: VolatileCell<u32>,
    rcgcadc: VolatileCell<u32>,
    rcgcacmp: VolatileCell<u32>,
    rcgcpwm: VolatileCell<u32>,
    rcgcqei: VolatileCell<u32>,
    _reserved32: [u32; 4],
    rcgceeprom: VolatileCell<u32>,
    _reserved33: [u32; 6],
    rcgcccm: VolatileCell<u32>,
    _reserved34: [u32; 9],
    rcgcemac: VolatileCell<u32>,
    _reserved35: [u32; 24],
    scgcwd: VolatileCell<u32>,
    scgctimer: VolatileCell<u32>,
    scgcgpio: VolatileCell<u32>,
    scgcdma: VolatileCell<u32>,
    scgcepi: VolatileCell<u32>,
    scgchib: VolatileCell<u32>,
    scgcuart: VolatileCell<u32>,
    scgcssi: VolatileCell<u32>,
    scgci2c: VolatileCell<u32>,
    _reserved36: [u32; 1],
    scgcusb: VolatileCell<u32>,
    _reserved37: [u32; 1],
    scgcephy: VolatileCell<u32>,
    scgccan: VolatileCell<u32>,
    scgcadc: VolatileCell<u32>,
    scgcacmp: VolatileCell<u32>,
    scgcpwm: VolatileCell<u32>,
    scgcqei: VolatileCell<u32>,
    _reserved38: [u32; 4],
    scgceeprom: VolatileCell<u32>,
    _reserved39: [u32; 6],
    scgcccm: VolatileCell<u32>,
    _reserved40: [u32; 9],
    scgcemac: VolatileCell<u32>,
    _reserved41: [u32; 24],
    dcgcwd: VolatileCell<u32>,
    dcgctimer: VolatileCell<u32>,
    dcgcgpio: VolatileCell<u32>,
    dcgcdma: VolatileCell<u32>,
    dcgcepi: VolatileCell<u32>,
    dcgchib: VolatileCell<u32>,
    dcgcuart: VolatileCell<u32>,
    dcgcssi: VolatileCell<u32>,
    dcgci2c: VolatileCell<u32>,
    _reserved42: [u32; 1],
    dcgcusb: VolatileCell<u32>,
    _reserved43: [u32; 1],
    dcgcephy: VolatileCell<u32>,
    dcgccan: VolatileCell<u32>,
    dcgcadc: VolatileCell<u32>,
    dcgcacmp: VolatileCell<u32>,
    dcgcpwm: VolatileCell<u32>,
    dcgcqei: VolatileCell<u32>,
    _reserved44: [u32; 4],
    dcgceeprom: VolatileCell<u32>,
    _reserved45: [u32; 6],
    dcgcccm: VolatileCell<u32>,
    _reserved46: [u32; 9],
    dcgcemac: VolatileCell<u32>,
    _reserved47: [u32; 24],
    pcwd: VolatileCell<u32>,
    pctimer: VolatileCell<u32>,
    pcgpio: VolatileCell<u32>,
    pcdma: VolatileCell<u32>,
    pcepi: VolatileCell<u32>,
    pchib: VolatileCell<u32>,
    pcuart: VolatileCell<u32>,
    pcssi: VolatileCell<u32>,
    pci2c: VolatileCell<u32>,
    _reserved48: [u32; 1],
    pcusb: VolatileCell<u32>,
    _reserved49: [u32; 1],
    pcephy: VolatileCell<u32>,
    pccan: VolatileCell<u32>,
    pcadc: VolatileCell<u32>,
    pcacmp: VolatileCell<u32>,
    pcpwm: VolatileCell<u32>,
    pcqei: VolatileCell<u32>,
    _reserved50: [u32; 4],
    pceeprom: VolatileCell<u32>,
    _reserved51: [u32; 6],
    pcccm: VolatileCell<u32>,
    _reserved52: [u32; 9],
    pcemac: VolatileCell<u32>,
    _reserved53: [u32; 24],
    prwd: VolatileCell<u32>,
    prtimer: VolatileCell<u32>,
    prgpio: VolatileCell<u32>,
    prdma: VolatileCell<u32>,
    prepi: VolatileCell<u32>,
    prhib: VolatileCell<u32>,
    pruart: VolatileCell<u32>,
    prssi: VolatileCell<u32>,
    pri2c: VolatileCell<u32>,
    _reserved54: [u32; 1],
    prusb: VolatileCell<u32>,
    _reserved55: [u32; 1],
    prephy: VolatileCell<u32>,
    prcan: VolatileCell<u32>,
    pradc: VolatileCell<u32>,
    pracmp: VolatileCell<u32>,
    prpwm: VolatileCell<u32>,
    prqei: VolatileCell<u32>,
    _reserved56: [u32; 4],
    preeprom: VolatileCell<u32>,
    _reserved57: [u32; 6],
    prccm: VolatileCell<u32>,
    _reserved58: [u32; 9],
    premac: VolatileCell<u32>,
}

#[derive(Copy, Clone, Debug)]
pub enum OscillatorFrequency {
    /// 25 MHz external oscillator
    Frequency25MHz,
}

#[derive(Copy, Clone, Debug)]
pub enum SystemClockSource {
    PioscAt16MHz,

    PllPioscAt120MHz,

    PllMoscAt120MHz,

    Mosc { frequency: OscillatorFrequency },
}

const BASE_ADDRESS: usize = 0x400FE000;

pub struct SystemControl {
    registers: *mut Registers,
    /// Frequency at which the system clock is running.
    system_frequency: Cell<u32>,
    /// Clock source configuration
    system_clock_source: Cell<SystemClockSource>,
}

pub static mut PSYSCTLM: SystemControl = SystemControl {
    registers: BASE_ADDRESS as *mut Registers,
    system_frequency: Cell::new(16000000),
    system_clock_source: Cell::new(SystemClockSource::PioscAt16MHz),
};

impl SystemControl {
    /// Sets up the system clock. This should be called as one of the first
    /// lines in the `reset_handler` within the platform's `main.rs`.
    pub unsafe fn setup_system_clock(&self, clock_source: SystemClockSource) {
        self.system_clock_source.set(clock_source);

        match clock_source {
            SystemClockSource::PioscAt16MHz => {
                // no configurations necessary, already running off the PIOSC
                self.system_frequency.set(16000000);
            }

            SystemClockSource::PllPioscAt120MHz => {
                configure_internal_oscillator_pll();
                self.system_frequency.set(120000000);
            }

            SystemClockSource::PllMoscAt120MHz => {
                configure_external_oscillator_pll();
                self.system_frequency.set(120000000);
            }
            SystemClockSource::Mosc { frequency } => {
                configure_external_oscillator();
                match frequency {
                    OscillatorFrequency::Frequency25MHz => self.system_frequency.set(25000000),
                };
            }
        }
    }
}

unsafe fn configure_internal_oscillator_pll() {
    let regs: &Registers = &*PSYSCTLM.registers;

    regs.rsclkcfg.set(0x00000000);

    regs.pllfreq1.set(0x0);
    regs.pllfreq0.set(0x1E | 0x00800000);
    regs.memtim0
        .set(0x00000190 | 0x01900000 | 0x5 | (0x5 << 16));
    regs.rsclkcfg.set(regs.rsclkcfg.get() | (0x40000000));

    while regs.pllstat.get() & (1) == (0) {}

    regs.rsclkcfg
        .set(regs.rsclkcfg.get() | 0x10000000 | 0x80000000 | 0x3);
}

unsafe fn configure_external_oscillator() {
    let regs: &Registers = &*PSYSCTLM.registers;

    regs.moscctl.set(0x10);
    while regs.ris.get() & (1 << 8) == (0) {}

    regs.rsclkcfg.set(regs.rsclkcfg.get() | (0x00300000));
    regs.memtim0
        .set(0x00000090 | 0x00900000 | 0x1 | (0x1 << 16));

    regs.rsclkcfg.set(regs.rsclkcfg.get() | 0x80000000);
}

unsafe fn configure_external_oscillator_pll() {
    let regs: &Registers = &*PSYSCTLM.registers;

    regs.moscctl.set(0x10); // OSCRNG
    while regs.ris.get() & (1 << 8) == (0) {}

    regs.rsclkcfg.set(regs.rsclkcfg.get() | (0x03300000));

    regs.pllfreq1.set(0x4);
    regs.pllfreq0.set(0x60 | 0x00800000);
    regs.memtim0
        .set(0x00000190 | 0x01900000 | 0x5 | (0x5 << 16));
    regs.rsclkcfg.set(regs.rsclkcfg.get() | (0x40000000));

    while regs.pllstat.get() & (1) == (0) {}

    regs.rsclkcfg.set(regs.rsclkcfg.get() | 0x90000003);
}

pub fn get_system_frequency() -> u32 {
    unsafe { PSYSCTLM.system_frequency.get() }
}

pub unsafe fn enable_clock(clock: Clock) {
    let regs: &Registers = &*PSYSCTLM.registers;
    match clock {
        Clock::TIMER(c) => regs.rcgctimer.set(regs.rcgctimer.get() | 1 << (c as u32)),
        Clock::GPIO(c) => regs.rcgcgpio.set(regs.rcgcgpio.get() | 1 << (c as u32)),
        Clock::UART(c) => regs.rcgcuart.set(regs.rcgcuart.get() | 1 << (c as u32)),
    }
}
