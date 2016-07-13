/// Stub implementation of a systick timer since the NRF doesn't have the
/// Cortex-M0 Systick. This will need to be replaced with one of the other
/// timers on the NRF, or maybe we don't care if only one process will ever run
/// on the NRF51


static mut VAL : usize = 0;

pub unsafe fn reset() {
    VAL = 0;
}

pub unsafe fn set_timer(val: usize) {
    VAL = val;
}

pub unsafe fn enable(_: bool) {
}

pub unsafe fn overflowed() -> bool {
    false
}

pub unsafe fn value() -> usize {
    VAL
}
