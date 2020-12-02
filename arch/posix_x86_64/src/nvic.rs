const NUM_INTERRUPTS: usize = 1;

pub const SYSTICK: usize = 0;

static mut PENDING_INTERRUPTS: [bool; NUM_INTERRUPTS] = [false; NUM_INTERRUPTS];

pub unsafe fn clear_interrupt(interrupt: usize) {
    PENDING_INTERRUPTS[interrupt] = false;
}

pub unsafe fn set_interrupt(interrupt: usize) {
    PENDING_INTERRUPTS[interrupt] = true;
}

pub unsafe fn has_pending() -> bool {
    for i in 0..NUM_INTERRUPTS {
        if PENDING_INTERRUPTS[i] {
            return true;
        }
    }
    false
}
