/// Stub implementation of a systick timer since the NRF doesn't have the
/// Cortex-M0 Systick. This will need to be replaced with one of the other
/// timers on the NRF, or maybe we don't care if only one process will ever run
/// on the NRF51


use common::VolatileCell;
use core::mem;


struct CLOCK {
    pub tasks_hfclkstart    : VolatileCell<u32>,
    pub tasks_hfclkstop     : VolatileCell<u32>,
    pub tasks_lfclkstart    : VolatileCell<u32>,
    pub tasks_lfclkstop     : VolatileCell<u32>,
    pub tasks_cal           : VolatileCell<u32>,
    pub tasks_cstart        : VolatileCell<u32>,
    pub tasks_cstop         : VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 57],
    pub events_hfclkstarted : VolatileCell<u32>,
    pub events_lfclkstarted : VolatileCell<u32>,
    pub done                : VolatileCell<u32>,
    pub ctto                : VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 125],
    pub intenset            : VolatileCell<u32>,
    pub intenclr            : VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 63],
    pub hfclkrun            : VolatileCell<u32>,
    pub hfclkstat           : VolatileCell<u32>,
    _reserved4: [VolatileCell<u32>; 1],
    pub lfclkrun            : VolatileCell<u32>,
    pub lfclkstat           : VolatileCell<u32>,
    pub lfclksrccopy        : VolatileCell<u32>,
    _reserved5: [VolatileCell<u32>; 62],
    pub lfclksrc            : VolatileCell<u32>,
    _reserved6: [VolatileCell<u32>; 7],
    pub ctiv                : VolatileCell<u32>,
    _reserved7: [VolatileCell<u32>; 5],
    pub xtalfreq            : VolatileCell<u32>,
}


struct RTC {
    pub tasks_start: VolatileCell<u32>,
    pub tasks_stop: VolatileCell<u32>,
    pub tasks_clear: VolatileCell<u32>,
    pub tasks_trigovrflw: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 60],
    pub events_tick: VolatileCell<u32>,
    pub events_ovrflw: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 14],
    pub events_compare: [VolatileCell<u32>; 4],
    _reserved3: [VolatileCell<u32>; 108],
    pub inten:    VolatileCell<u32>,
    pub intenset: VolatileCell<u32>,
    pub intenclr: VolatileCell<u32>,
    _reserved4: [VolatileCell<u32>; 13],
    pub evten: VolatileCell<u32>,
    pub evtenset: VolatileCell<u32>,
    pub evtenclr: VolatileCell<u32>,
    _reserved5: [VolatileCell<u32>; 110],
    pub counter: VolatileCell<u32>,
    pub prescaler: VolatileCell<u32>,
    _reserved6: [VolatileCell<u32>; 13],
    pub cc: [VolatileCell<u32>; 4],
    _reserved7: [VolatileCell<u32>; 683],
    pub power: VolatileCell<u32>,
}
const CLOCK_BASE: usize = 0x40000000;
const RTC0_BASE: usize =  0x4000B000; 

static mut VAL: usize = 0;

#[allow(non_snake_case)]
fn RTC0() -> &'static RTC {
        unsafe { mem::transmute(RTC0_BASE as usize) }
}

#[allow(non_snake_case)]
fn CLOCK() -> &'static CLOCK {
        unsafe { mem::transmute(CLOCK_BASE as usize) }
}

pub unsafe fn reset() {
    RTC0().tasks_clear.set(1);
}

pub unsafe fn set_timer(val: usize) {
    // Clock is 32768Hz
    // Each tick is 30.51us, approximate as 30.5
    let ticks: u32 = (val as u32 * 10) / 305;
    RTC0().cc[0].set(ticks); // Compare value
    VAL = val
}

pub unsafe fn enable(_: bool) {
    // Cribbed from mynewt; some of this overkill but be safe
    CLOCK().xtalfreq.set(0);
    CLOCK().tasks_lfclkstop.set(1);
    CLOCK().events_lfclkstarted.set(0);
    CLOCK().lfclksrc.set(0);
    CLOCK().tasks_lfclkstart.set(1);
    while CLOCK().events_lfclkstarted.get() == 0  {} 
    RTC0().prescaler.set(0);
    RTC0().tasks_stop.set(1);
    RTC0().tasks_clear.set(1);
    RTC0().evtenclr.set(0xffffffff);
    RTC0().intenclr.set(0xffffffff);
    RTC0().tasks_start.set(1);
}

pub unsafe fn overflowed() -> bool {
//    value() >= (RTC0().cc[0].get() as usize)
    false
}

#[allow(unused_variables)]
pub unsafe fn value() -> usize {
    // Clock is 32768Hz
    // Each tick is 30.51us, approximate as 30.5
    let counter = RTC0().counter.get() as usize;
    let val = counter * 305 / 10;
    // Keep this hardcoded value because counter seems to
    // not work correctly
    VAL * 1000
}

