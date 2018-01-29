//! Implementation of the Backup System Control Interface (BSCIF) peripheral.

use kernel::common::VolatileCell;

#[repr(C)]
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

const BSCIF_BASE: usize = 0x400F0400;
static mut BSCIF: *mut BscifRegisters = BSCIF_BASE as *mut BscifRegisters;

/// Setup the internal 32kHz RC oscillator.
pub unsafe fn enable_rc32k() {
    let bscif_rc32kcr = (*BSCIF).rc32kcr.get();
    // Unlock the BSCIF::RC32KCR register
    (*BSCIF).unlock.set(0xAA000024);
    // Write the BSCIF::RC32KCR register.
    // Enable the generic clock source, the temperature compensation, and the
    // 32k output.
    (*BSCIF)
        .rc32kcr
        .set(bscif_rc32kcr | (1 << 2) | (1 << 1) | (1 << 0));
    // Wait for it to be ready, although it feels like this won't do anything
    while (*BSCIF).rc32kcr.get() & (1 << 0) == 0 {}

    // Load magic calibration value for the 32KHz RC oscillator
    //
    // Unlock the BSCIF::RC32KTUNE register
    (*BSCIF).unlock.set(0xAA000028);
    // Write the BSCIF::RC32KTUNE register
    (*BSCIF).rc32ktune.set(0x001d0015);
}
