//! ARM System Control Block
//!
//! <http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CIHFDJCA.html>

use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;

#[repr(C)]
struct ScbRegisters {
    cpuid: VolatileCell<u32>,
    icsr: VolatileCell<u32>,
    vtor: VolatileCell<*const ()>,
    aircr: VolatileCell<u32>,
    scr: VolatileCell<u32>,
    ccr: VolatileCell<u32>,
    shp: [VolatileCell<u32>; 3],
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

const SCB: StaticRef<ScbRegisters> = unsafe { StaticRef::new(0xE000ED00 as *const ScbRegisters) };

/// Allow the core to go into deep sleep on WFI.
///
/// The specific definition of "deep sleep" is chip specific.
pub unsafe fn set_sleepdeep() {
    let scr = SCB.scr.get();
    SCB.scr.set(scr | 1 << 2);
}

/// Do not allow the core to go into deep sleep on WFI.
///
/// The specific definition of "deep sleep" is chip specific.
pub unsafe fn unset_sleepdeep() {
    let scr = SCB.scr.get();
    SCB.scr.set(scr & !(1 << 2));
}

/// Software reset using the ARM System Control Block
pub unsafe fn reset() {
    let aircr = SCB.aircr.get();
    let reset = (0x5FA << 16) | (aircr & (0x7 << 8)) | (1 << 2);
    SCB.aircr.set(reset);
}

/// relocate interrupt vector table
pub unsafe fn set_vector_table_offset(offset: *const ()) {
    SCB.vtor.set(offset);
}
