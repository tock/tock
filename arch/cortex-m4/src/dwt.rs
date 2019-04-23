use kernel::common::registers::{ReadWrite};
use kernel::common::StaticRef;

struct DWTRegisters {
    ctrl: ReadWrite<u32>,
    cycnt: ReadWrite<u32>,
}

struct DBGRegisters {
    demcr: ReadWrite<u32>,
}

const DWT: StaticRef<DWTRegisters> = unsafe { StaticRef::new(0xE0001000 as *const _) };
const DEMCR: StaticRef<ReadWrite<u32>> = unsafe { StaticRef::new(0xE000EDFC as *const _) };

pub unsafe fn reset_timer() {
    DEMCR.set(DEMCR.get() | 0x01000000);
    DWT.cycnt.set(0); // reset the counter
    DWT.ctrl.set(0); // disable counter;
}

pub unsafe fn start_timer() {
    DWT.ctrl.set(1); // enable counter;
}

pub unsafe fn stop_timer() {
    DWT.ctrl.set(0); // disable counter;
}

pub unsafe fn get_time() -> u32 {
    DWT.cycnt.get()
}

pub unsafe fn bench<F: FnOnce()>(f: F) -> u32 {
    reset_timer();
    start_timer();
    f();
    stop_timer();
    get_time()
}
