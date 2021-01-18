use nix::sys::signal::{self, SigSet, SigmaskHow};

extern "C" {
    fn pause() -> isize;
}

/// Waits for a signal to be received
pub fn wfi() {
    let mut current_signals = SigSet::empty();
    let masked_signals = SigSet::empty();
    signal::sigprocmask(
        SigmaskHow::SIG_SETMASK,
        Some(&masked_signals),
        Some(&mut current_signals),
    )
    .unwrap();

    unsafe {
        pause();
    }

    signal::sigprocmask(SigmaskHow::SIG_SETMASK, Some(&current_signals), None).unwrap();
}

pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let mut current_signals = SigSet::empty();
    let masked_signals = SigSet::all();

    // Mask Signals
    signal::sigprocmask(
        SigmaskHow::SIG_SETMASK,
        Some(&masked_signals),
        Some(&mut current_signals),
    )
    .unwrap();
    let res = f();

    // Unmask Signals
    signal::sigprocmask(SigmaskHow::SIG_SETMASK, Some(&current_signals), None).unwrap();
    res
}

#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        llvm_asm!("nop" :::: "volatile");
    }
}
