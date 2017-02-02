// Based on: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CIHFDJCA.html

use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
struct ScbRegisters {
    cpuid: VolatileCell<u32>,
    icsr: VolatileCell<u32>,
    vtor: VolatileCell<u32>,
    aircr: VolatileCell<u32>,
    scr: VolatileCell<u32>,
    ccr: VolatileCell<u32>,
    shp: [VolatileCell<u32>; 12],
    shcsr: VolatileCell<u32>,
    cfsr: VolatileCell<u32>,
    hfsr: VolatileCell<u32>,
    dfsr: VolatileCell<u32>,
    mmfar: VolatileCell<u32>,
    bfar: VolatileCell<u32>,
    afsr: VolatileCell<u32>,
    pfr: [VolatileCell<u32>; 2],
    dfr: VolatileCell<u32>,
    adr: VolatileCell<u32>,
    mmfr: [VolatileCell<u32>; 4],
    isar: [VolatileCell<u32>; 5],
    _reserved0: [u32; 5],
    cpacr: VolatileCell<u32>,
}

const SCB_BASE: usize = 0xE000ED00;

static mut SCB: *mut ScbRegisters = SCB_BASE as *mut ScbRegisters;

/// Software reset using the ARM System Control Block
pub unsafe fn reset() {
    let aircr = (*SCB).aircr.get();
    let reset = (0x5FA << 16) | (aircr & (0x7 << 8)) | (1 << 2);
    (*SCB).aircr.set(reset);
}
